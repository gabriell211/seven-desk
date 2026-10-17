#![forbid(unsafe_code)]

//! Development signaling API. This process does not stream a screen and must
//! NOT be published as a production service until persistence, TLS, per-tenant
//! authorization, quotas and incident operations have been validated.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use seven_protocol::{
    Capability, DeviceId, SessionId, SessionMachine, SessionRequest, SessionState,
};
use seven_security::{NonceWindow, RequestProof, verify};
use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

const MAX_INVITATION_MS: u64 = 5 * 60 * 1_000;
const PRESENCE_TTL_MS: u64 = 90_000;
const MAX_PAYLOAD_SIZE: usize = 64 * 1_024;

#[derive(Default)]
pub struct AppState {
    devices: RwLock<HashMap<DeviceId, u64>>,
    sessions: Mutex<HashMap<SessionId, SessionMachine>>,
    nonce_window: NonceWindow,
}

#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    Forbidden,
    Invalid,
    NotFound,
    Conflict,
    Unavailable,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "authentication_failed"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "not_authorized"),
            Self::Invalid => (StatusCode::BAD_REQUEST, "invalid_request"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::Conflict => (StatusCode::CONFLICT, "invalid_session_state"),
            Self::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, "temporary_unavailable"),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn authenticated(
    state: &AppState,
    headers: &HeaderMap,
    method: &str,
    path: &str,
    body: &[u8],
) -> Result<DeviceId, ApiError> {
    if body.len() > MAX_PAYLOAD_SIZE {
        return Err(ApiError::Invalid);
    }
    let get = |name| -> Result<String, ApiError> {
        Ok(headers
            .get(name)
            .ok_or(ApiError::Unauthorized)?
            .to_str()
            .map_err(|_| ApiError::Unauthorized)?
            .to_owned())
    };
    let proof = RequestProof {
        public_key_hex: get("x-seven-key")?,
        signature_hex: get("x-seven-signature")?,
        nonce: get("x-seven-nonce")?
            .parse()
            .map_err(|_| ApiError::Unauthorized)?,
        timestamp_ms: get("x-seven-timestamp")?
            .parse()
            .map_err(|_| ApiError::Unauthorized)?,
    };
    let (device, _) = verify(&proof, method, path, body, now_ms(), &state.nonce_window)
        .map_err(|_| ApiError::Unauthorized)?;
    Ok(device)
}

fn known(state: &AppState, device: DeviceId) -> Result<(), ApiError> {
    let devices = state.devices.read().map_err(|_| ApiError::Unavailable)?;
    if devices
        .get(&device)
        .is_some_and(|expires_at| *expires_at > now_ms())
    {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}

#[derive(Serialize)]
struct Status<'a> {
    status: &'a str,
    release: &'a str,
    protocol: u16,
    production_ready: bool,
}

async fn health() -> Json<Status<'static>> {
    Json(Status {
        status: "ok",
        release: env!("CARGO_PKG_VERSION"),
        protocol: seven_protocol::PROTOCOL_VERSION,
        production_ready: false,
    })
}

#[derive(Serialize)]
struct Registered {
    device_id: DeviceId,
    status: &'static str,
}

async fn register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Registered>, ApiError> {
    if body.as_ref() != b"{}" {
        return Err(ApiError::Invalid);
    }
    let device = authenticated(&state, &headers, "POST", "/v1/devices", &body)?;
    let now = now_ms();
    let mut devices = state.devices.write().map_err(|_| ApiError::Unavailable)?;
    // Keep only recently authenticated devices in the development registry.
    devices.retain(|_, expires_at| *expires_at > now);
    devices.insert(device, now.saturating_add(PRESENCE_TTL_MS));
    Ok(Json(Registered {
        device_id: device,
        status: "registered",
    }))
}

#[derive(Serialize)]
struct PresenceStatus {
    device_id: DeviceId,
    online: bool,
}

/// Only a currently registered device can refresh its own presence.
/// An expired device must re-register with a fresh signed proof.
async fn heartbeat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<PresenceStatus>, ApiError> {
    if body.as_ref() != b"{}" {
        return Err(ApiError::Invalid);
    }
    let actor = authenticated(&state, &headers, "POST", "/v1/devices/heartbeat", &body)?;
    let now = now_ms();
    let mut devices = state.devices.write().map_err(|_| ApiError::Unavailable)?;
    if devices
        .get(&actor)
        .is_none_or(|expires_at| *expires_at <= now)
    {
        return Err(ApiError::Unauthorized);
    }
    devices.insert(actor, now.saturating_add(PRESENCE_TTL_MS));
    Ok(Json(PresenceStatus {
        device_id: actor,
        online: true,
    }))
}

/// Authenticated viewers can check a target's recent presence without listing
/// every registered device or learning whether an unknown ID ever existed.
async fn presence(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<PresenceStatus>, ApiError> {
    let path = format!("/v1/devices/{id}/presence");
    let actor = authenticated(&state, &headers, "GET", &path, &[])?;
    known(&state, actor)?;
    let device = DeviceId(id);
    let online = state
        .devices
        .read()
        .map_err(|_| ApiError::Unavailable)?
        .get(&device)
        .is_some_and(|expires_at| *expires_at > now_ms());
    Ok(Json(PresenceStatus {
        device_id: device,
        online,
    }))
}

async fn create_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    let operator = authenticated(&state, &headers, "POST", "/v1/sessions", &body)?;
    known(&state, operator)?;
    let request: SessionRequest = serde_json::from_slice(&body).map_err(|_| ApiError::Invalid)?;
    if request.operator != operator
        || request.expires_at_ms > now_ms().saturating_add(MAX_INVITATION_MS)
    {
        return Err(ApiError::Forbidden);
    }
    known(&state, request.host)?;
    let mut machine = SessionMachine::default();
    machine
        .request(request.clone(), now_ms())
        .map_err(|_| ApiError::Invalid)?;
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    if sessions.contains_key(&request.session_id) {
        return Err(ApiError::Conflict);
    }
    sessions.insert(request.session_id, machine);
    Ok((
        StatusCode::CREATED,
        Json(SessionResponse {
            session_id: request.session_id,
            status: "awaiting_consent",
        }),
    ))
}

#[derive(Serialize)]
struct SessionResponse {
    session_id: SessionId,
    status: &'static str,
}

#[derive(Serialize)]
struct PendingInvitation {
    session_id: SessionId,
    operator: DeviceId,
    requested: BTreeSet<Capability>,
    expires_at_ms: u64,
}

/// The authenticated host can inspect only its own outstanding invitations.
/// Reading the inbox never grants permissions or starts a remote session.
async fn pending_invitations(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<PendingInvitation>>, ApiError> {
    let host = authenticated(&state, &headers, "GET", "/v1/sessions/inbox", &[])?;
    known(&state, host)?;

    let now = now_ms();
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    let mut pending = Vec::new();

    for machine in sessions.values_mut() {
        // Expire invitations before exposure; rejected and ended requests are
        // deliberately excluded even when their participants are retained.
        machine.expire(now);
        if let SessionState::AwaitingConsent(request) = machine.state()
            && request.host == host
        {
            pending.push(PendingInvitation {
                session_id: request.session_id,
                operator: request.operator,
                requested: request.requested.clone(),
                expires_at_ms: request.expires_at_ms,
            });
        }
    }

    pending.sort_by_key(|item| item.expires_at_ms);
    Ok(Json(pending))
}

#[derive(Serialize)]
struct ActiveAuthorization {
    session_id: SessionId,
    operator: DeviceId,
    granted: BTreeSet<Capability>,
    expires_at_ms: u64,
}

/// The host always retains the ability to inspect and end its authorizations,
/// including after its UI restarts. No other device can enumerate them.
async fn active_authorizations(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ActiveAuthorization>>, ApiError> {
    let host = authenticated(&state, &headers, "GET", "/v1/sessions/host/active", &[])?;
    known(&state, host)?;
    let now = now_ms();
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    let mut active = Vec::new();
    for machine in sessions.values_mut() {
        machine.expire(now);
        if let SessionState::Active {
            request,
            granted,
            session_deadline_ms,
        } = machine.state()
            && request.host == host
        {
            active.push(ActiveAuthorization {
                session_id: request.session_id,
                operator: request.operator,
                granted: granted.clone(),
                expires_at_ms: *session_deadline_ms,
            });
        }
    }
    active.sort_by_key(|entry| entry.expires_at_ms);
    Ok(Json(active))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    approve: bool,
    #[serde(default)]
    granted: BTreeSet<Capability>,
}

async fn decide(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<SessionResponse>, ApiError> {
    let path = format!("/v1/sessions/{id}/decision");
    let actor = authenticated(&state, &headers, "POST", &path, &body)?;
    known(&state, actor)?;
    let decision: Decision = serde_json::from_slice(&body).map_err(|_| ApiError::Invalid)?;
    let session_id = SessionId(id);
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    let machine = sessions.get_mut(&session_id).ok_or(ApiError::NotFound)?;
    let result = if decision.approve {
        machine.accept(actor, decision.granted, now_ms())
    } else {
        machine.reject(actor)
    };
    result.map_err(|_| ApiError::Forbidden)?;
    Ok(Json(SessionResponse {
        session_id,
        status: if decision.approve {
            "active"
        } else {
            "rejected"
        },
    }))
}

async fn terminate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    let path = format!("/v1/sessions/{id}");
    let actor = authenticated(&state, &headers, "DELETE", &path, &[])?;
    known(&state, actor)?;
    let session_id = SessionId(id);
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    let machine = sessions.get_mut(&session_id).ok_or(ApiError::NotFound)?;
    machine.terminate(actor).map_err(|_| ApiError::Forbidden)?;
    Ok(Json(SessionResponse {
        session_id,
        status: "ended",
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revocation {
    capability: Capability,
}

async fn revoke(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<SessionResponse>, ApiError> {
    let path = format!("/v1/sessions/{id}/revoke");
    let actor = authenticated(&state, &headers, "POST", &path, &body)?;
    known(&state, actor)?;
    let input: Revocation = serde_json::from_slice(&body).map_err(|_| ApiError::Invalid)?;
    let session_id = SessionId(id);
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    let machine = sessions.get_mut(&session_id).ok_or(ApiError::NotFound)?;
    machine
        .revoke(actor, input.capability)
        .map_err(|_| ApiError::Forbidden)?;
    Ok(Json(SessionResponse {
        session_id,
        status: if matches!(machine.state(), SessionState::Ended) {
            "ended"
        } else {
            "active"
        },
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PublicState {
    AwaitingConsent,
    Active,
    Rejected,
    Ended,
}

async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<PublicState>, ApiError> {
    let path = format!("/v1/sessions/{id}");
    let actor = authenticated(&state, &headers, "GET", &path, &[])?;
    known(&state, actor)?;
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    let machine = sessions.get_mut(&SessionId(id)).ok_or(ApiError::NotFound)?;
    let member = machine
        .participants()
        .is_some_and(|(host, operator)| host == actor || operator == actor);
    if !member {
        return Err(ApiError::Forbidden);
    }
    machine.expire(now_ms());
    let value = match machine.state() {
        SessionState::AwaitingConsent(_) => PublicState::AwaitingConsent,
        SessionState::Active { .. } => PublicState::Active,
        SessionState::Rejected => PublicState::Rejected,
        _ => PublicState::Ended,
    };
    Ok(Json(value))
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/healthz",
            get(health).layer(
                CorsLayer::new()
                    .allow_origin([
                        HeaderValue::from_static("http://127.0.0.1:1420"),
                        HeaderValue::from_static("http://localhost:1420"),
                        HeaderValue::from_static("http://tauri.localhost"),
                        HeaderValue::from_static("tauri://localhost"),
                    ])
                    .allow_methods([Method::GET]),
            ),
        )
        .route("/v1/devices", post(register))
        .route("/v1/devices/heartbeat", post(heartbeat))
        .route("/v1/devices/{id}/presence", get(presence))
        .route("/v1/sessions", post(create_session))
        .route("/v1/sessions/inbox", get(pending_invitations))
        .route("/v1/sessions/host/active", get(active_authorizations))
        .route("/v1/sessions/{id}", get(get_session).delete(terminate))
        .route("/v1/sessions/{id}/decision", post(decide))
        .route("/v1/sessions/{id}/revoke", post(revoke))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_marks_server_as_not_production_ready() {
        let response = router(Arc::new(AppState::default()))
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        assert!(
            std::str::from_utf8(&body)
                .unwrap()
                .contains("\"production_ready\":false")
        );
    }

    #[tokio::test]
    async fn registration_requires_signed_proof() {
        let response = router(Arc::new(AppState::default()))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/devices")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    use ed25519_dalek::{Signer, SigningKey};
    use seven_security::{device_id, message_to_sign};

    fn signed_request(key: &SigningKey, method: &str, path: &str, body: Vec<u8>) -> Request<Body> {
        let nonce = Uuid::new_v4();
        let timestamp = now_ms();
        let proof = message_to_sign(method, path, &body, nonce, timestamp).unwrap();
        let signature = key.sign(&proof);
        Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .header("x-seven-key", hex::encode(key.verifying_key().as_bytes()))
            .header("x-seven-signature", hex::encode(signature.to_bytes()))
            .header("x-seven-nonce", nonce.to_string())
            .header("x-seven-timestamp", timestamp.to_string())
            .body(Body::from(body))
            .unwrap()
    }

    #[tokio::test]
    async fn authenticated_operator_cannot_approve_and_host_can_revoke() {
        let service = router(Arc::new(AppState::default()));
        let operator = SigningKey::from_bytes(&[15_u8; 32]);
        let host = SigningKey::from_bytes(&[22_u8; 32]);

        for key in [&operator, &host] {
            let res = service
                .clone()
                .oneshot(signed_request(key, "POST", "/v1/devices", b"{}".to_vec()))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        let session_id = SessionId(Uuid::new_v4());
        let body = serde_json::to_vec(&SessionRequest {
            protocol_version: seven_protocol::PROTOCOL_VERSION,
            session_id,
            operator: device_id(&operator.verifying_key()),
            host: device_id(&host.verifying_key()),
            requested: BTreeSet::from([Capability::Screen, Capability::Input]),
            expires_at_ms: now_ms() + 30_000,
        })
        .unwrap();
        let response = service
            .clone()
            .oneshot(signed_request(&operator, "POST", "/v1/sessions", body))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let decision_path = format!("/v1/sessions/{}/decision", session_id.0);
        let grant = serde_json::to_vec(&serde_json::json!({
            "approve": true,
            "granted": ["screen", "input"]
        }))
        .unwrap();

        let unauthorized = service
            .clone()
            .oneshot(signed_request(
                &operator,
                "POST",
                &decision_path,
                grant.clone(),
            ))
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::FORBIDDEN);

        let approved = service
            .clone()
            .oneshot(signed_request(&host, "POST", &decision_path, grant))
            .await
            .unwrap();
        assert_eq!(approved.status(), StatusCode::OK);

        let revoke_path = format!("/v1/sessions/{}/revoke", session_id.0);
        let revocation = serde_json::to_vec(&serde_json::json!({
            "capability": "screen"
        }))
        .unwrap();
        let blocked = service
            .clone()
            .oneshot(signed_request(
                &operator,
                "POST",
                &revoke_path,
                revocation.clone(),
            ))
            .await
            .unwrap();
        assert_eq!(blocked.status(), StatusCode::FORBIDDEN);

        let ended = service
            .clone()
            .oneshot(signed_request(&host, "POST", &revoke_path, revocation))
            .await
            .unwrap();
        assert_eq!(ended.status(), StatusCode::OK);

        let session_path = format!("/v1/sessions/{}", session_id.0);
        let final_state = service
            .clone()
            .oneshot(signed_request(&operator, "GET", &session_path, Vec::new()))
            .await
            .unwrap();
        assert_eq!(final_state.status(), StatusCode::OK);
        let payload = to_bytes(final_state.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        assert_eq!(payload.as_ref(), b"\"ended\"");
    }

    #[tokio::test]
    async fn pending_inbox_is_scoped_to_target_host_and_respects_rejection() {
        let service = router(Arc::new(AppState::default()));
        let operator = SigningKey::from_bytes(&[23_u8; 32]);
        let host = SigningKey::from_bytes(&[24_u8; 32]);
        let other = SigningKey::from_bytes(&[25_u8; 32]);

        for key in [&operator, &host, &other] {
            let response = service
                .clone()
                .oneshot(signed_request(key, "POST", "/v1/devices", b"{}".to_vec()))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        let session_id = SessionId(Uuid::new_v4());
        let invitation = SessionRequest {
            protocol_version: seven_protocol::PROTOCOL_VERSION,
            session_id,
            operator: device_id(&operator.verifying_key()),
            host: device_id(&host.verifying_key()),
            requested: BTreeSet::from([Capability::Screen]),
            expires_at_ms: now_ms() + 30_000,
        };

        let response = service
            .clone()
            .oneshot(signed_request(
                &operator,
                "POST",
                "/v1/sessions",
                serde_json::to_vec(&invitation).unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        for key in [&operator, &other] {
            let response = service
                .clone()
                .oneshot(signed_request(key, "GET", "/v1/sessions/inbox", Vec::new()))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
                .await
                .unwrap();
            assert_eq!(body.as_ref(), b"[]");
        }

        let response = service
            .clone()
            .oneshot(signed_request(
                &host,
                "GET",
                "/v1/sessions/inbox",
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        let pending: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let entries = pending.as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["session_id"], session_id.0.to_string());
        assert_eq!(entries[0]["operator"], invitation.operator.0.to_string());
        assert_eq!(entries[0]["requested"], serde_json::json!(["screen"]));

        let decision = format!("/v1/sessions/{}/decision", session_id.0);
        let response = service
            .clone()
            .oneshot(signed_request(
                &host,
                "POST",
                &decision,
                serde_json::to_vec(&serde_json::json!({"approve": false})).unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = service
            .clone()
            .oneshot(signed_request(
                &host,
                "GET",
                "/v1/sessions/inbox",
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"[]");
    }

    #[tokio::test]
    async fn pending_inbox_rejects_unsigned_access_and_expired_requests() {
        let state = Arc::new(AppState::default());
        let host = SigningKey::from_bytes(&[26_u8; 32]);
        let operator = SigningKey::from_bytes(&[27_u8; 32]);
        let service = router(state.clone());

        let response = service
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/sessions/inbox")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response = service
            .clone()
            .oneshot(signed_request(
                &host,
                "GET",
                "/v1/sessions/inbox",
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let registration = service
            .clone()
            .oneshot(signed_request(&host, "POST", "/v1/devices", b"{}".to_vec()))
            .await
            .unwrap();
        assert_eq!(registration.status(), StatusCode::OK);

        // Build an already-expired invitation directly in test state.
        let expired = SessionRequest {
            protocol_version: seven_protocol::PROTOCOL_VERSION,
            session_id: SessionId(Uuid::new_v4()),
            operator: device_id(&operator.verifying_key()),
            host: device_id(&host.verifying_key()),
            requested: BTreeSet::from([Capability::Screen]),
            expires_at_ms: 1,
        };
        let mut machine = SessionMachine::default();
        machine.request(expired.clone(), 0).unwrap();
        state
            .sessions
            .lock()
            .unwrap()
            .insert(expired.session_id, machine);

        let response = service
            .clone()
            .oneshot(signed_request(
                &host,
                "GET",
                "/v1/sessions/inbox",
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"[]");
        assert!(matches!(
            state
                .sessions
                .lock()
                .unwrap()
                .get(&expired.session_id)
                .unwrap()
                .state(),
            SessionState::Ended
        ));
    }

    #[tokio::test]
    async fn signed_presence_expires_and_blocks_invitation_to_offline_host() {
        let state = Arc::new(AppState::default());
        let service = router(state.clone());
        let operator = SigningKey::from_bytes(&[31_u8; 32]);
        let host = SigningKey::from_bytes(&[32_u8; 32]);

        for key in [&operator, &host] {
            let result = service
                .clone()
                .oneshot(signed_request(key, "POST", "/v1/devices", b"{}".to_vec()))
                .await
                .unwrap();
            assert_eq!(result.status(), StatusCode::OK);
        }

        let host_id = device_id(&host.verifying_key());
        let url = format!("/v1/devices/{}/presence", host_id.0);
        let response = service
            .clone()
            .oneshot(signed_request(&operator, "GET", &url, Vec::new()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        let presence: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(presence["online"], true);

        let heartbeat = service
            .clone()
            .oneshot(signed_request(
                &host,
                "POST",
                "/v1/devices/heartbeat",
                b"{}".to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(heartbeat.status(), StatusCode::OK);

        // Simulate the presence TTL expiring without sleeping in the test.
        state.devices.write().unwrap().insert(host_id, 0);
        let response = service
            .clone()
            .oneshot(signed_request(&operator, "GET", &url, Vec::new()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        let presence: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(presence["online"], false);

        let invite = SessionRequest {
            protocol_version: seven_protocol::PROTOCOL_VERSION,
            session_id: SessionId(Uuid::new_v4()),
            operator: device_id(&operator.verifying_key()),
            host: host_id,
            requested: BTreeSet::from([Capability::Screen]),
            expires_at_ms: now_ms() + 30_000,
        };
        let response = service
            .clone()
            .oneshot(signed_request(
                &operator,
                "POST",
                "/v1/sessions",
                serde_json::to_vec(&invite).unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let stale_heartbeat = service
            .clone()
            .oneshot(signed_request(
                &host,
                "POST",
                "/v1/devices/heartbeat",
                b"{}".to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(stale_heartbeat.status(), StatusCode::UNAUTHORIZED);

        let registered_again = service
            .clone()
            .oneshot(signed_request(&host, "POST", "/v1/devices", b"{}".to_vec()))
            .await
            .unwrap();
        assert_eq!(registered_again.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn presence_rejects_unsigned_and_unregistered_callers() {
        let service = router(Arc::new(AppState::default()));
        let key = SigningKey::from_bytes(&[33_u8; 32]);
        let target = DeviceId(Uuid::new_v4());
        let url = format!("/v1/devices/{}/presence", target.0);

        let unsigned = service
            .clone()
            .oneshot(Request::builder().uri(&url).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(unsigned.status(), StatusCode::UNAUTHORIZED);

        let unknown = service
            .clone()
            .oneshot(signed_request(&key, "GET", &url, Vec::new()))
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::UNAUTHORIZED);

        let response = service
            .clone()
            .oneshot(signed_request(&key, "POST", "/v1/devices", b"{}".to_vec()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = service
            .clone()
            .oneshot(signed_request(&key, "GET", &url, Vec::new()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["online"], false);
    }

    #[tokio::test]
    async fn active_authorizations_are_visible_only_to_host_and_remain_revocable() {
        let service = router(Arc::new(AppState::default()));
        let operator = SigningKey::from_bytes(&[51_u8; 32]);
        let host = SigningKey::from_bytes(&[52_u8; 32]);
        let outsider = SigningKey::from_bytes(&[53_u8; 32]);
        let listing_path = "/v1/sessions/host/active";

        for key in [&operator, &host, &outsider] {
            let registered = service
                .clone()
                .oneshot(signed_request(key, "POST", "/v1/devices", b"{}".to_vec()))
                .await
                .unwrap();
            assert_eq!(registered.status(), StatusCode::OK);
        }

        let session_id = SessionId(Uuid::new_v4());
        let invitation = SessionRequest {
            protocol_version: seven_protocol::PROTOCOL_VERSION,
            session_id,
            operator: device_id(&operator.verifying_key()),
            host: device_id(&host.verifying_key()),
            requested: BTreeSet::from([Capability::Screen, Capability::Input]),
            expires_at_ms: now_ms() + 30_000,
        };
        let requested = service
            .clone()
            .oneshot(signed_request(
                &operator,
                "POST",
                "/v1/sessions",
                serde_json::to_vec(&invitation).unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(requested.status(), StatusCode::CREATED);

        let decision = format!("/v1/sessions/{}/decision", session_id.0);
        let approved = service
            .clone()
            .oneshot(signed_request(
                &host,
                "POST",
                &decision,
                br#"{"approve":true,"granted":["screen"]}"#.to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(approved.status(), StatusCode::OK);

        let unsigned = service
            .clone()
            .oneshot(Request::builder().uri(listing_path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(unsigned.status(), StatusCode::UNAUTHORIZED);

        for key in [&operator, &outsider] {
            let response = service
                .clone()
                .oneshot(signed_request(key, "GET", listing_path, vec![]))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let raw = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
                .await
                .unwrap();
            assert_eq!(raw.as_ref(), b"[]");
        }

        let response = service
            .clone()
            .oneshot(signed_request(&host, "GET", listing_path, vec![]))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let raw = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        let sessions: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(sessions.as_array().unwrap().len(), 1);
        assert_eq!(sessions[0]["session_id"], session_id.0.to_string());
        assert_eq!(sessions[0]["operator"], invitation.operator.0.to_string());
        assert_eq!(sessions[0]["granted"], serde_json::json!(["screen"]));

        let terminated = service
            .clone()
            .oneshot(signed_request(
                &host,
                "DELETE",
                &format!("/v1/sessions/{}", session_id.0),
                vec![],
            ))
            .await
            .unwrap();
        assert_eq!(terminated.status(), StatusCode::OK);

        let response = service
            .clone()
            .oneshot(signed_request(&host, "GET", listing_path, vec![]))
            .await
            .unwrap();
        let raw = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE)
            .await
            .unwrap();
        assert_eq!(raw.as_ref(), b"[]");
    }

    #[tokio::test]
    async fn sessions_do_not_allow_unsigned_requests() {
        let response = router(Arc::new(AppState::default()))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/sessions")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

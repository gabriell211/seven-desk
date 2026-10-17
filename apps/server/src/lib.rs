#![forbid(unsafe_code)]

//! Development signaling API. This process does not stream a screen and must
//! NOT be published as a production service until persistence, TLS, per-tenant
//! authorization, quotas and incident operations have been validated.

use axum::{
    Router, Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use seven_protocol::{Capability, DeviceId, SessionId, SessionMachine, SessionRequest, SessionState};
use seven_security::{NonceWindow, RequestProof, verify};
use std::{collections::{BTreeSet, HashMap}, sync::{Arc, Mutex, RwLock}, time::{SystemTime, UNIX_EPOCH}};
use uuid::Uuid;

const MAX_INVITATION_MS: u64 = 5 * 60 * 1_000;
const MAX_PAYLOAD_SIZE: usize = 64 * 1_024;

#[derive(Default)]
pub struct AppState {
    devices: RwLock<BTreeSet<Uuid>>,
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
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
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
        Ok(headers.get(name).ok_or(ApiError::Unauthorized)?
            .to_str().map_err(|_| ApiError::Unauthorized)?.to_owned())
    };
    let proof = RequestProof {
        public_key_hex: get("x-seven-key")?,
        signature_hex: get("x-seven-signature")?,
        nonce: get("x-seven-nonce")?.parse().map_err(|_| ApiError::Unauthorized)?,
        timestamp_ms: get("x-seven-timestamp")?.parse().map_err(|_| ApiError::Unauthorized)?,
    };
    let (device, _) = verify(&proof, method, path, body, now_ms(), &state.nonce_window)
        .map_err(|_| ApiError::Unauthorized)?;
    Ok(device)
}

fn known(state: &AppState, device: DeviceId) -> Result<(), ApiError> {
    let devices = state.devices.read().map_err(|_| ApiError::Unavailable)?;
    if devices.contains(&device.0) { Ok(()) } else { Err(ApiError::Unauthorized) }
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
    state.devices.write().map_err(|_| ApiError::Unavailable)?.insert(device.0);
    Ok(Json(Registered { device_id: device, status: "registered" }))
}

async fn create_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    let operator = authenticated(&state, &headers, "POST", "/v1/sessions", &body)?;
    known(&state, operator)?;
    let request: SessionRequest = serde_json::from_slice(&body).map_err(|_| ApiError::Invalid)?;
    if request.operator != operator || request.expires_at_ms > now_ms().saturating_add(MAX_INVITATION_MS) {
        return Err(ApiError::Forbidden);
    }
    known(&state, request.host)?;
    let mut machine = SessionMachine::default();
    machine.request(request.clone(), now_ms()).map_err(|_| ApiError::Invalid)?;
    let mut sessions = state.sessions.lock().map_err(|_| ApiError::Unavailable)?;
    if sessions.contains_key(&request.session_id) {
        return Err(ApiError::Conflict);
    }
    sessions.insert(request.session_id, machine);
    Ok((StatusCode::CREATED, Json(SessionResponse { session_id: request.session_id, status: "awaiting_consent" })))
}

#[derive(Serialize)]
struct SessionResponse {
    session_id: SessionId,
    status: &'static str,
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
        status: if decision.approve { "active" } else { "rejected" },
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
    Ok(Json(SessionResponse { session_id, status: "ended" }))
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
    let member = match machine.state() {
        SessionState::AwaitingConsent(r) | SessionState::Active { request: r, .. } => {
            r.operator == actor || r.host == actor
        }
        _ => false,
    };
    if !member { return Err(ApiError::Forbidden); }
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
        .route("/healthz", get(health))
        .route("/v1/devices", post(register))
        .route("/v1/sessions", post(create_session))
        .route("/v1/sessions/{id}", get(get_session).delete(terminate))
        .route("/v1/sessions/{id}/decision", post(decide))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::{Body, to_bytes}, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_marks_server_as_not_production_ready() {
        let response = router(Arc::new(AppState::default()))
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), MAX_PAYLOAD_SIZE).await.unwrap();
        assert!(std::str::from_utf8(&body).unwrap().contains("\"production_ready\":false"));
    }

    #[tokio::test]
    async fn registration_requires_signed_proof() {
        let response = router(Arc::new(AppState::default()))
            .oneshot(Request::builder().method("POST").uri("/v1/devices")
                .header("content-type", "application/json")
                .body(Body::from("{}")).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn sessions_do_not_allow_unsigned_requests() {
        let response = router(Arc::new(AppState::default()))
            .oneshot(Request::builder().method("POST").uri("/v1/sessions")
                .header("content-type", "application/json")
                .body(Body::from("{}")).unwrap())
            .await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

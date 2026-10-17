//! Signed session negotiation for the loopback-only development control plane.
//! No video, input, file transfer or unattended access is implemented here.
use reqwest::Method;
use serde::{Deserialize, Serialize};
use seven_identity::{Identity, SystemKeyStore};
use seven_protocol::{Capability, DeviceId, PROTOCOL_VERSION, SessionId, SessionRequest};
use std::{
    collections::BTreeSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const API_BASE: &str = "http://127.0.0.1:43821";
const INVITATION_TTL_MS: u64 = 120_000;
const MAX_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Serialize, Deserialize)]
pub struct SessionReceipt {
    session_id: Uuid,
    status: String,
}

#[derive(Serialize, Deserialize)]
pub struct PendingInvitation {
    session_id: Uuid,
    operator: Uuid,
    requested: BTreeSet<Capability>,
    expires_at_ms: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ActiveAuthorization {
    session_id: Uuid,
    operator: Uuid,
    granted: BTreeSet<Capability>,
    expires_at_ms: u64,
}

fn timestamp_ms() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Relógio local inválido".to_string())?
        .as_millis() as u64)
}

fn invitation(operator: Uuid, host: Uuid, now: u64) -> Result<SessionRequest, String> {
    if operator == host {
        return Err("Não é permitido solicitar suporte para o próprio dispositivo".into());
    }
    Ok(SessionRequest {
        protocol_version: PROTOCOL_VERSION,
        session_id: SessionId(Uuid::new_v4()),
        operator: DeviceId(operator),
        host: DeviceId(host),
        requested: BTreeSet::from([Capability::Screen]),
        expires_at_ms: now.saturating_add(INVITATION_TTL_MS),
    })
}

fn parse_session_id(raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|_| "Identificador de sessão inválido".to_string())
}

/// Signed native calls are restricted to paths constructed by this module.
/// Neither React nor a remote HTTP response can choose arbitrary request paths.
async fn signed_request(method: Method, path: String, body: Vec<u8>) -> Result<Vec<u8>, String> {
    let method_name = method.as_str().to_owned();
    let sign_path = path.clone();
    let sign_body = body.clone();
    let proof = tauri::async_runtime::spawn_blocking(move || {
        let identity = Identity::load(&SystemKeyStore)
            .map_err(|_| "Cofre de credenciais indisponível".to_string())?
            .ok_or_else(|| "Ative uma identidade de dispositivo antes de solicitar sessões".to_string())?;
        identity
            .sign(
                &method_name,
                &sign_path,
                &sign_body,
                Uuid::new_v4(),
                timestamp_ms()?,
            )
            .map_err(|_| "Não foi possível assinar a requisição".to_string())
    })
    .await
    .map_err(|_| "Falha ao acessar credenciais nativas".to_string())??;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .no_proxy()
        .build()
        .map_err(|_| "Falha ao abrir conexão local".to_string())?;
    let response = client
        .request(method, format!("{API_BASE}{path}"))
        .header("x-seven-key", proof.public_key_hex)
        .header("x-seven-signature", proof.signature_hex)
        .header("x-seven-nonce", proof.nonce.to_string())
        .header("x-seven-timestamp", proof.timestamp_ms.to_string())
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|_| "Servidor de desenvolvimento indisponível".to_string())?;

    let code = response.status();
    if !code.is_success() {
        return Err(format!("Solicitação recusada pelo servidor (HTTP {})", code.as_u16()));
    }

    if response.content_length().is_some_and(|len| len > MAX_RESPONSE_BYTES as u64) {
        return Err("Resposta do servidor excede o limite permitido".to_string());
    }
    let raw = response
        .bytes()
        .await
        .map_err(|_| "Erro ao ler resposta do servidor".to_string())?;
    if raw.len() > MAX_RESPONSE_BYTES {
        return Err("Resposta do servidor excede o limite permitido".to_string());
    }
    Ok(raw.to_vec())
}

async fn read_device_id() -> Result<Uuid, String> {
    tauri::async_runtime::spawn_blocking(|| {
        Ok::<Uuid, String>(
            Identity::load(&SystemKeyStore)
                .map_err(|_| "Cofre de credenciais indisponível".to_string())?
                .ok_or_else(|| "Ative sua identidade local antes".to_string())?
                .id()
                .0,
        )
    })
    .await
    .map_err(|_| "Falha ao consultar identidade".to_string())?
}

#[tauri::command]
pub async fn create_session_request(target_id: String) -> Result<SessionReceipt, String> {
    let target = Uuid::parse_str(target_id.trim())
        .map_err(|_| "Digite o ID completo e válido do dispositivo remoto".to_string())?;
    let request = invitation(read_device_id().await?, target, timestamp_ms()?)?;
    let data = serde_json::to_vec(&request).map_err(|_| "Falha ao criar convite".to_string())?;
    let raw = signed_request(Method::POST, "/v1/sessions".to_owned(), data).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para convite".to_string())
}

#[tauri::command]
pub async fn list_pending_invitations() -> Result<Vec<PendingInvitation>, String> {
    let raw = signed_request(Method::GET, "/v1/sessions/inbox".to_owned(), vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para convites".to_string())
}

/// Approval always grants screen negotiation only; it never enables input.
/// The session is metadata-only until separately authorized media is built.
#[tauri::command]
pub async fn list_active_authorizations() -> Result<Vec<ActiveAuthorization>, String> {
    let raw = signed_request(Method::GET, "/v1/sessions/host/active".to_owned(), vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para autorizações ativas".to_string())
}

#[tauri::command]
pub async fn decide_session(session_id: String, approve: bool) -> Result<SessionReceipt, String> {
    let id = parse_session_id(&session_id)?;
    let body = if approve {
        br#"{"approve":true,"granted":["screen"]}"#.to_vec()
    } else {
        br#"{"approve":false,"granted":[]}"#.to_vec()
    };
    let path = format!("/v1/sessions/{id}/decision");
    let raw = signed_request(Method::POST, path, body).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para decisão".to_string())
}

#[tauri::command]
pub async fn session_status(session_id: String) -> Result<String, String> {
    let id = parse_session_id(&session_id)?;
    let path = format!("/v1/sessions/{id}");
    let raw = signed_request(Method::GET, path, vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Estado de sessão inválido".to_string())
}

#[tauri::command]
pub async fn end_session(session_id: String) -> Result<SessionReceipt, String> {
    let id = parse_session_id(&session_id)?;
    let path = format!("/v1/sessions/{id}");
    let raw = signed_request(Method::DELETE, path, vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para encerramento".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invitation_only_requests_screen_and_has_short_expiry() {
        let operator = Uuid::new_v4();
        let host = Uuid::new_v4();
        let request = invitation(operator, host, 1_000).unwrap();
        assert_eq!(request.operator.0, operator);
        assert_eq!(request.host.0, host);
        assert_eq!(request.expires_at_ms, 121_000);
        assert_eq!(request.requested, BTreeSet::from([Capability::Screen]));
        assert_eq!(request.protocol_version, PROTOCOL_VERSION);
        assert!(request.validate(1_000).is_ok());
    }

    #[test]
    fn self_invitation_and_invalid_session_ids_fail_closed() {
        let id = Uuid::new_v4();
        assert!(invitation(id, id, 1_000).is_err());
        assert!(parse_session_id("not-a-uuid").is_err());
        assert_eq!(parse_session_id(&id.to_string()), Ok(id));
    }
}

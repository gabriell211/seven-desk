//! Signed session negotiation for the loopback-only development control plane.
//! No video, input, file transfer or unattended access is implemented here.
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use seven_identity::{Identity, SystemKeyStore};
use seven_protocol::{Capability, DeviceId, MouseEvent, PROTOCOL_VERSION, SessionId, SessionRequest};
use std::{
    collections::BTreeSet,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const INVITATION_TTL_MS: u64 = 120_000;
const MAX_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_FRAME_BYTES: usize = 2 * 1024 * 1024;

// The native process defaults to refusing mouse injection on every launch.
static ENABLED_HOST_MOUSE: Mutex<Option<Uuid>> = Mutex::new(None);

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

#[derive(Serialize, Deserialize)]
pub struct ActiveOperatorSession {
    session_id: Uuid,
    host: Uuid,
    granted: BTreeSet<Capability>,
    expires_at_ms: u64,
}

fn timestamp_ms() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Relógio local inválido".to_string())?
        .as_millis() as u64)
}

fn invitation(operator: Uuid, host: Uuid, now: u64, request_mouse: bool) -> Result<SessionRequest, String> {
    if operator == host {
        return Err("Não é permitido solicitar suporte para o próprio dispositivo".into());
    }
    Ok(SessionRequest {
        protocol_version: PROTOCOL_VERSION,
        session_id: SessionId(Uuid::new_v4()),
        operator: DeviceId(operator),
        host: DeviceId(host),
        requested: if request_mouse {
            BTreeSet::from([Capability::Screen, Capability::Input])
        } else {
            BTreeSet::from([Capability::Screen])
        },
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

    let upload_frame = method == Method::POST && path.ends_with("/frame");
    let max_response = if method == Method::GET && path.ends_with("/frame") {
        MAX_FRAME_BYTES
    } else {
        MAX_RESPONSE_BYTES
    };
    let base = crate::configured_server_url()?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .no_proxy()
        .build()
        .map_err(|_| "Falha ao abrir conexão local".to_string())?;
    let mut response = client
        .request(method, format!("{base}{path}"))
        .header("x-seven-key", proof.public_key_hex)
        .header("x-seven-signature", proof.signature_hex)
        .header("x-seven-nonce", proof.nonce.to_string())
        .header("x-seven-timestamp", proof.timestamp_ms.to_string())
        .header("content-type", if upload_frame {
            "image/png"
        } else {
            "application/json"
        })
        .body(body)
        .send()
        .await
        .map_err(|_| "Servidor de desenvolvimento indisponível".to_string())?;

    let code = response.status();
    if !code.is_success() {
        return Err(format!("Solicitação recusada pelo servidor (HTTP {})", code.as_u16()));
    }

    if response.content_length().is_some_and(|len| len > max_response as u64) {
        return Err("Resposta do servidor excede o limite permitido".to_string());
    }
    let mut raw = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Erro ao ler resposta do servidor".to_string())?
    {
        if chunk.len() > max_response.saturating_sub(raw.len()) {
            return Err("Resposta do servidor excede o limite permitido".to_string());
        }
        raw.extend_from_slice(&chunk);
    }
    Ok(raw)
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
pub async fn create_session_request(target_id: String, request_mouse: bool) -> Result<SessionReceipt, String> {
    let target = Uuid::parse_str(target_id.trim())
        .map_err(|_| "Digite o ID completo e válido do dispositivo remoto".to_string())?;
    let request = invitation(read_device_id().await?, target, timestamp_ms()?, request_mouse)?;
    let data = serde_json::to_vec(&request).map_err(|_| "Falha ao criar convite".to_string())?;
    let raw = signed_request(Method::POST, "/v1/sessions".to_owned(), data).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para convite".to_string())
}

#[tauri::command]
pub async fn list_pending_invitations() -> Result<Vec<PendingInvitation>, String> {
    let raw = signed_request(Method::GET, "/v1/sessions/inbox".to_owned(), vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para convites".to_string())
}

/// List active grants for the authenticated host only.
#[tauri::command]
pub async fn list_active_authorizations() -> Result<Vec<ActiveAuthorization>, String> {
    let raw = signed_request(Method::GET, "/v1/sessions/host/active".to_owned(), vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para autorizações ativas".to_string())
}

#[tauri::command]
pub async fn list_active_operator_sessions() -> Result<Vec<ActiveOperatorSession>, String> {
    let raw = signed_request(
        Method::GET,
        "/v1/sessions/operator/active".to_owned(),
        vec![],
    )
    .await?;
    serde_json::from_slice(&raw)
        .map_err(|_| "Resposta inválida ao consultar sessões do operador".to_string())
}

#[tauri::command]
pub async fn decide_session(
    session_id: String,
    approve: bool,
    grant_mouse: bool,
) -> Result<SessionReceipt, String> {
    let id = parse_session_id(&session_id)?;
    let body = if approve && grant_mouse {
        br#"{"approve":true,"granted":["screen","input"]}"#.to_vec()
    } else if approve {
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
    {
        let mut current = ENABLED_HOST_MOUSE
            .lock()
            .map_err(|_| "Falha no controle local do mouse".to_string())?;
        if *current == Some(id) {
            *current = None;
        }
    }
    let path = format!("/v1/sessions/{id}");
    let raw = signed_request(Method::DELETE, path, vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para encerramento".to_string())
}

fn host_mouse_is_enabled(id: Uuid) -> Result<bool, String> {
    Ok(*ENABLED_HOST_MOUSE
        .lock()
        .map_err(|_| "Falha no controle local do mouse".to_string())?
        == Some(id))
}

/// Operator may only submit bounded mouse events signed with its native key.
/// The backend independently checks that Input and Screen were granted.
#[tauri::command]
pub async fn send_remote_mouse(session_id: String, event: MouseEvent) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    event.validate().map_err(|_| "Evento de mouse inválido".to_string())?;
    let body = serde_json::to_vec(&event)
        .map_err(|_| "Falha ao codificar o evento do mouse".to_string())?;
    signed_request(Method::POST, format!("/v1/sessions/{id}/input"), body).await?;
    Ok(())
}

/// Foreground host opt-in is separate from granting the session capability.
/// On every new activation, drain any previously queued events so old clicks
/// cannot execute after the host changes its mind.
#[tauri::command]
pub async fn configure_host_mouse(session_id: String, enabled: bool) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    if !enabled {
        let mut current = ENABLED_HOST_MOUSE
            .lock()
            .map_err(|_| "Falha no controle local do mouse".to_string())?;
        if *current == Some(id) {
            *current = None;
        }
        return Ok(());
    }
    if !cfg!(target_os = "windows") {
        return Err("Controle nativo de mouse disponível apenas no Windows".to_string());
    }
    let authorizations = list_active_authorizations().await?;
    if !authorizations.iter().any(|item| {
        item.session_id == id
            && item.granted.contains(&Capability::Input)
            && item.granted.contains(&Capability::Screen)
    }) {
        return Err("O host não concedeu permissão para controle do mouse".to_string());
    }
    // Drain a prior input batch before this user action activates injection.
    let path = format!("/v1/sessions/{id}/input");
    let raw = signed_request(Method::GET, path, vec![]).await?;
    let _: Vec<MouseEvent> = serde_json::from_slice(&raw)
        .map_err(|_| "Resposta inválida do canal de entrada".to_string())?;
    *ENABLED_HOST_MOUSE
        .lock()
        .map_err(|_| "Falha no controle local do mouse".to_string())? = Some(id);
    Ok(())
}

fn normalized_pointer(value: u16, dimension: i32) -> i32 {
    let limit = dimension.saturating_sub(1).max(0) as i64;
    ((i64::from(value) * limit) / 10_000) as i32
}

#[cfg(target_os = "windows")]
fn apply_mouse_batch(id: Uuid, events: Vec<MouseEvent>) -> Result<usize, String> {
    use enigo::{Axis, Button, Coordinate, Direction, Enigo, Mouse, Settings};
    let mut backend = Enigo::new(&Settings::default())
        .map_err(|_| "Não foi possível inicializar entrada nativa do Windows".to_string())?;
    let (width, height) = backend
        .main_display()
        .map_err(|_| "Não foi possível identificar o monitor principal".to_string())?;
    if width < 2 || height < 2 {
        return Err("Resolução inválida do monitor principal".to_string());
    }
    let mut applied = 0;
    for event in events {
        // A UI host can stop input between individual events in a batch.
        if !host_mouse_is_enabled(id)? {
            break;
        }
        event.validate().map_err(|_| "Evento de entrada fora dos limites".to_string())?;
        let (x, y) = match event {
            MouseEvent::Move { x, y }
            | MouseEvent::Click { x, y, .. }
            | MouseEvent::Scroll { x, y, .. } => (x, y),
        };
        backend
            .move_mouse(
                normalized_pointer(x, width),
                normalized_pointer(y, height),
                Coordinate::Abs,
            )
            .map_err(|_| "O Windows recusou o movimento do cursor".to_string())?;
        match event {
            MouseEvent::Move { .. } => {}
            MouseEvent::Click { button, .. } => {
                let button = match button {
                    seven_protocol::MouseButton::Left => Button::Left,
                    seven_protocol::MouseButton::Right => Button::Right,
                    seven_protocol::MouseButton::Middle => Button::Middle,
                };
                backend
                    .button(button, Direction::Click)
                    .map_err(|_| "O Windows recusou o clique".to_string())?;
            }
            MouseEvent::Scroll { steps, .. } => {
                backend
                    .scroll(i32::from(steps), Axis::Vertical)
                    .map_err(|_| "O Windows recusou a rolagem".to_string())?;
            }
        }
        applied += 1;
    }
    Ok(applied)
}

#[cfg(not(target_os = "windows"))]
fn apply_mouse_batch(_id: Uuid, _events: Vec<MouseEvent>) -> Result<usize, String> {
    Err("Controle remoto de mouse disponível somente no Windows".to_string())
}

/// The host retrieves and applies events only while the local user has
/// explicitly enabled the input path. No keyboard, UAC or elevated desktop.
#[tauri::command]
pub async fn poll_host_mouse(session_id: String) -> Result<usize, String> {
    let id = parse_session_id(&session_id)?;
    if !host_mouse_is_enabled(id)? {
        return Err("Controle do mouse desativado no host".to_string());
    }
    let path = format!("/v1/sessions/{id}/input");
    let raw = signed_request(Method::GET, path, vec![]).await?;
    let events: Vec<MouseEvent> = serde_json::from_slice(&raw)
        .map_err(|_| "Lote de entrada inválido".to_string())?;
    if events.len() > 32 || events.iter().any(|event| event.validate().is_err()) {
        return Err("Lote de entrada excedeu os limites permitidos".to_string());
    }
    if events.is_empty() || !host_mouse_is_enabled(id)? {
        return Ok(0);
    }
    // Check grant again after network delivery, before touching OS input.
    let authorized = list_active_authorizations().await?.iter().any(|item| {
        item.session_id == id
            && item.granted.contains(&Capability::Input)
            && item.granted.contains(&Capability::Screen)
    });
    if !authorized {
        return Err("Permissão de mouse revogada pelo host".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || apply_mouse_batch(id, events))
        .await
        .map_err(|_| "Falha na thread nativa de mouse".to_string())?
}

/// Host-only on/off switch for the frame channel; changing to false clears
/// server-side pixels and rejects any late upload from an older capture.
#[tauri::command]
pub async fn set_screen_delivery(session_id: String, enabled: bool) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    let path = format!("/v1/sessions/{id}/screen-delivery");
    let payload = if enabled {
        br#"{"enabled":true}"#.to_vec()
    } else {
        br#"{"enabled":false}"#.to_vec()
    };
    let raw = signed_request(Method::POST, path, payload).await?;
    let response: serde_json::Value = serde_json::from_slice(&raw)
        .map_err(|_| "Resposta inválida ao configurar compartilhamento".to_string())?;
    if response.get("enabled").and_then(serde_json::Value::as_bool) != Some(enabled) {
        return Err("O servidor não confirmou o estado de compartilhamento".to_string());
    }
    Ok(())
}

/// Manually capture a single host frame from the primary monitor (Windows).
/// Capture cannot run until the host has actively approved the specific session.
#[cfg(target_os = "windows")]
fn capture_png() -> Result<Vec<u8>, String> {
    use image::{DynamicImage, ImageFormat, imageops::FilterType};
    use std::io::Cursor;

    let monitors = xcap::Monitor::all()
        .map_err(|_| "Não foi possível listar os monitores deste dispositivo".to_string())?;
    let selected = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);
    let monitor = monitors
        .get(selected)
        .ok_or_else(|| "Nenhum monitor disponível para captura".to_string())?;
    let screenshot = monitor
        .capture_image()
        .map_err(|_| "O Windows não autorizou a captura da tela".to_string())?;

    let source = DynamicImage::ImageRgba8(screenshot);
    // Keep first-stage frame transfers strictly bounded, without storing pixels.
    // Progressive downscaling also ensures high-resolution desktops don't silently
    // cause an unbounded upload or memory spike.
    for width in [1_600_u32, 1_280, 1_024, 800, 640, 480, 320] {
        let scaled = source.resize(width, width * 2, FilterType::Triangle);
        let mut cursor = Cursor::new(Vec::new());
        scaled
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|_| "Falha ao codificar o quadro da tela".to_string())?;
        let png = cursor.into_inner();
        if png.len() <= MAX_FRAME_BYTES {
            return Ok(png);
        }
    }
    Err("O quadro excedeu o limite de transferência permitido".to_string())
}

#[cfg(not(target_os = "windows"))]
fn capture_png() -> Result<Vec<u8>, String> {
    Err("Captura manual ainda não está disponível neste sistema operacional".to_string())
}

/// This command is an explicit action from the local host, not a background
/// stream. The server rechecks host identity and Screen permission upon upload.
#[tauri::command]
pub async fn publish_manual_screen_frame(session_id: String) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    let authorizations = list_active_authorizations().await?;
    if !authorizations.iter().any(|item| {
        item.session_id == id && item.granted.contains(&Capability::Screen)
    }) {
        return Err("A sessão não possui autorização ativa para visualizar a tela".to_string());
    }
    let frame = tauri::async_runtime::spawn_blocking(capture_png)
        .await
        .map_err(|_| "Falha ao realizar a captura nativa".to_string())??;
    let path = format!("/v1/sessions/{id}/frame");
    signed_request(Method::POST, path, frame).await?;
    Ok(())
}

/// The operator must be authenticated and still have Screen permission.
#[tauri::command]
pub async fn fetch_authorized_screen_frame(session_id: String) -> Result<String, String> {
    let id = parse_session_id(&session_id)?;
    let path = format!("/v1/sessions/{id}/frame");
    let png = signed_request(Method::GET, path, vec![]).await?;
    if png.len() > MAX_FRAME_BYTES || !png.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("O servidor retornou um quadro inválido".to_string());
    }
    Ok(BASE64.encode(png))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invitation_only_requests_screen_and_has_short_expiry() {
        let operator = Uuid::new_v4();
        let host = Uuid::new_v4();
        let request = invitation(operator, host, 1_000, false).unwrap();
        assert_eq!(request.operator.0, operator);
        assert_eq!(request.host.0, host);
        assert_eq!(request.expires_at_ms, 121_000);
        assert_eq!(request.requested, BTreeSet::from([Capability::Screen]));
        assert_eq!(request.protocol_version, PROTOCOL_VERSION);
        assert!(request.validate(1_000).is_ok());
    }

    #[test]
    fn mouse_is_never_requested_without_opt_in_and_coordinates_are_bounded() {
        let source = Uuid::new_v4();
        let dest = Uuid::new_v4();
        let view = invitation(source, dest, 100, false).unwrap();
        let control = invitation(source, dest, 100, true).unwrap();
        assert_eq!(view.requested, BTreeSet::from([Capability::Screen]));
        assert_eq!(
            control.requested,
            BTreeSet::from([Capability::Screen, Capability::Input])
        );
        assert_eq!(normalized_pointer(0, 1920), 0);
        assert_eq!(normalized_pointer(10_000, 1920), 1919);
        assert_eq!(normalized_pointer(5_000, 1080), 539);
        assert_eq!(normalized_pointer(8_000, 0), 0);
    }

    #[test]
    fn self_invitation_and_invalid_session_ids_fail_closed() {
        let id = Uuid::new_v4();
        assert!(invitation(id, id, 1_000, true).is_err());
        assert!(parse_session_id("not-a-uuid").is_err());
        assert_eq!(parse_session_id(&id.to_string()), Ok(id));
    }
}

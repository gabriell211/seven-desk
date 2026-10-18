//! Signed session negotiation for the loopback-only development control plane.
//! Experimental native screen, mouse and keyboard paths require host consent.
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use seven_identity::{Identity, SystemKeyStore};
use seven_protocol::SpecialKey;
use seven_protocol::{
    Capability, DeviceId, KeyboardEvent, MouseEvent, PROTOCOL_VERSION, SessionId, SessionRequest,
};
use std::time::Instant;
use std::{
    collections::BTreeSet,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::Emitter;
use uuid::Uuid;

const INVITATION_TTL_MS: u64 = 120_000;
const MAX_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_FRAME_BYTES: usize = 2 * 1024 * 1024;

// The native process defaults to refusing mouse injection on every launch.
static ENABLED_HOST_MOUSE: Mutex<Option<Uuid>> = Mutex::new(None);
static ENABLED_HOST_KEYBOARD: Mutex<Option<Uuid>> = Mutex::new(None);
static ACTIVE_MOUSE_PUMP: Mutex<Option<(Uuid, Arc<AtomicBool>)>> = Mutex::new(None);
static ACTIVE_KEYBOARD_PUMP: Mutex<Option<(Uuid, Arc<AtomicBool>)>> = Mutex::new(None);
// Local selection only. A secondary display is always view-only until input
// mapping across each OS virtual desktop has been validated.
static CAPTURE_MONITOR: Mutex<Option<(u32, bool)>> = Mutex::new(None);
static ACTIVE_CAPTURE: Mutex<Option<(Uuid, Arc<AtomicBool>)>> = Mutex::new(None);

#[derive(Clone, Serialize)]
struct CaptureStopped {
    session_id: Uuid,
    message: String,
}

#[derive(Clone, Serialize)]
struct HostInputStopped {
    session_id: Uuid,
    kind: &'static str,
    message: String,
}

fn stop_input_pump(pump: &Mutex<Option<(Uuid, Arc<AtomicBool>)>>, id: Uuid) {
    let current = pump.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((session, running)) = current.as_ref() {
        if *session == id {
            running.store(false, Ordering::Release);
        }
    }
}

fn stop_capture_for(id: Uuid) -> Result<(), String> {
    let active = ACTIVE_CAPTURE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if active.as_ref().is_some_and(|(current, _)| *current == id) {
        if let Some((_, running)) = active.as_ref() {
            running.store(false, Ordering::Release);
        }
    }
    Ok(())
}

fn stop_any_capture() -> Result<(), String> {
    let active = ACTIVE_CAPTURE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((_, running)) = active.as_ref() {
        running.store(false, Ordering::Release);
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct SessionReceipt {
    session_id: Uuid,
    status: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    sequence: u64,
    sender: Uuid,
    text: String,
    sent_at_ms: u64,
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

#[derive(Deserialize)]
struct PresenceResult {
    device_id: Uuid,
    online: bool,
}

#[derive(Serialize)]
pub struct HostMonitor {
    id: u32,
    name: String,
    width: u32,
    height: u32,
    is_primary: bool,
}

fn monitors_on_device() -> Result<Vec<HostMonitor>, String> {
    let monitors = xcap::Monitor::all()
        .map_err(|_| "Não foi possível listar os monitores deste dispositivo".to_string())?;
    monitors
        .iter()
        .map(|monitor| {
            Ok(HostMonitor {
                id: monitor.id().map_err(|_| "ID do monitor indisponível")?,
                name: monitor.name().map_err(|_| "Nome do monitor indisponível")?,
                width: monitor
                    .width()
                    .map_err(|_| "Largura do monitor indisponível")?,
                height: monitor
                    .height()
                    .map_err(|_| "Altura do monitor indisponível")?,
                is_primary: monitor
                    .is_primary()
                    .map_err(|_| "Estado do monitor indisponível")?,
            })
        })
        .collect()
}

#[tauri::command]
pub async fn list_host_monitors() -> Result<Vec<HostMonitor>, String> {
    tauri::async_runtime::spawn_blocking(monitors_on_device)
        .await
        .map_err(|_| "Falha ao consultar os monitores".to_string())?
}

#[tauri::command]
pub async fn select_host_monitor(monitor_id: u32) -> Result<HostMonitor, String> {
    // Selection starts by stopping input, even when the requested monitor was
    // unplugged. The host can opt in again after choosing a valid primary.
    *ENABLED_HOST_MOUSE
        .lock()
        .map_err(|_| "Falha no controle local do mouse".to_string())? = None;
    stop_any_capture()?;
    let monitors = list_host_monitors().await?;
    let selected = monitors
        .into_iter()
        .find(|monitor| monitor.id == monitor_id)
        .ok_or_else(|| "Monitor selecionado não está mais disponível".to_string())?;
    // Stop local input before changing the captured display. Enigo currently
    // maps absolute coordinates only to the primary display.
    *CAPTURE_MONITOR
        .lock()
        .map_err(|_| "Falha ao selecionar monitor".to_string())? =
        Some((selected.id, selected.is_primary));
    Ok(selected)
}

fn timestamp_ms() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Relógio local inválido".to_string())?
        .as_millis() as u64)
}

fn invitation(
    operator: Uuid,
    host: Uuid,
    now: u64,
    request_mouse: bool,
    request_keyboard: bool,
    request_chat: bool,
) -> Result<SessionRequest, String> {
    if operator == host {
        return Err("Não é permitido solicitar suporte para o próprio dispositivo".into());
    }
    let mut requested = BTreeSet::from([Capability::Screen]);
    if request_mouse {
        requested.insert(Capability::Input);
    }
    if request_keyboard {
        requested.insert(Capability::Keyboard);
    }
    if request_chat {
        requested.insert(Capability::Chat);
    }
    Ok(SessionRequest {
        protocol_version: PROTOCOL_VERSION,
        session_id: SessionId(Uuid::new_v4()),
        operator: DeviceId(operator),
        host: DeviceId(host),
        requested,
        expires_at_ms: now.saturating_add(INVITATION_TTL_MS),
    })
}

fn parse_session_id(raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|_| "Identificador de sessão inválido".to_string())
}

/// Signed native calls are restricted to paths constructed by this module.
/// Neither React nor a remote HTTP response can choose arbitrary request paths.
async fn signed_request(method: Method, path: String, body: Vec<u8>) -> Result<Vec<u8>, String> {
    let base = crate::configured_server_url()?;
    let method_name = method.as_str().to_owned();
    let sign_path = path.clone();
    let sign_body = body.clone();
    let proof = tauri::async_runtime::spawn_blocking(move || {
        let identity = Identity::load(&SystemKeyStore)
            .map_err(|_| "Cofre de credenciais indisponível".to_string())?
            .ok_or_else(|| {
                "Ative uma identidade de dispositivo antes de solicitar sessões".to_string()
            })?;
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
    let client =
        crate::server_config::http_client(if upload_frame && body.starts_with(b"\xff\xd8") {
            Duration::from_secs(2)
        } else {
            Duration::from_secs(5)
        })?;
    let mut response = client
        .request(method, format!("{base}{path}"))
        .header("x-seven-key", proof.public_key_hex)
        .header("x-seven-signature", proof.signature_hex)
        .header("x-seven-nonce", proof.nonce.to_string())
        .header("x-seven-timestamp", proof.timestamp_ms.to_string())
        .header(
            "content-type",
            if upload_frame {
                if body.starts_with(b"\xff\xd8") {
                    "image/jpeg"
                } else {
                    "image/png"
                }
            } else {
                "application/json"
            },
        )
        .body(body)
        .send()
        .await
        .map_err(|_| "Servidor de desenvolvimento indisponível".to_string())?;

    let code = response.status();
    if !code.is_success() {
        return Err(format!(
            "Solicitação recusada pelo servidor (HTTP {})",
            code.as_u16()
        ));
    }

    if response
        .content_length()
        .is_some_and(|len| len > max_response as u64)
    {
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
pub async fn create_session_request(
    target_id: String,
    request_mouse: bool,
    request_keyboard: bool,
    request_chat: bool,
) -> Result<SessionReceipt, String> {
    let target = Uuid::parse_str(target_id.trim())
        .map_err(|_| "Digite o ID completo e válido do dispositivo remoto".to_string())?;
    let request = invitation(
        read_device_id().await?,
        target,
        timestamp_ms()?,
        request_mouse,
        request_keyboard,
        request_chat,
    )?;
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
    serde_json::from_slice(&raw)
        .map_err(|_| "Resposta inválida para autorizações ativas".to_string())
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
    grant_keyboard: bool,
    grant_chat: bool,
) -> Result<SessionReceipt, String> {
    let id = parse_session_id(&session_id)?;
    let mut granted = BTreeSet::new();
    if approve {
        granted.insert(Capability::Screen);
        if grant_mouse {
            granted.insert(Capability::Input);
        }
        if grant_keyboard {
            granted.insert(Capability::Keyboard);
        }
        if grant_chat {
            granted.insert(Capability::Chat);
        }
    }
    let body = serde_json::to_vec(&serde_json::json!({"approve": approve, "granted": granted}))
        .map_err(|_| "Falha ao codificar a autorização".to_string())?;
    let path = format!("/v1/sessions/{id}/decision");
    let raw = signed_request(Method::POST, path, body).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para decisão".to_string())
}

/// Read signed, scoped presence. A saved contact never grants session access.
#[tauri::command]
pub async fn check_device_presence(device_id: String) -> Result<bool, String> {
    let id = Uuid::parse_str(device_id.trim()).map_err(|_| "ID do contato inválido".to_string())?;
    let path = format!("/v1/devices/{id}/presence");
    let raw = signed_request(Method::GET, path, vec![]).await?;
    let result: PresenceResult =
        serde_json::from_slice(&raw).map_err(|_| "Resposta inválida de presença".to_string())?;
    if result.device_id != id {
        return Err("Identidade de presença não corresponde à consulta".to_string());
    }
    Ok(result.online)
}

#[tauri::command]
pub async fn send_session_chat(session_id: String, text: String) -> Result<ChatMessage, String> {
    if text.trim().is_empty() || text.len() > 2_048 || text.chars().count() > 500 {
        return Err("Mensagem inválida ou muito longa".to_string());
    }
    let id = parse_session_id(&session_id)?;
    let body = serde_json::to_vec(&serde_json::json!({"text": text}))
        .map_err(|_| "Falha ao codificar mensagem".to_string())?;
    let raw = signed_request(Method::POST, format!("/v1/sessions/{id}/chat"), body).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida do chat".to_string())
}

#[tauri::command]
pub async fn list_session_chat(session_id: String) -> Result<Vec<ChatMessage>, String> {
    let id = parse_session_id(&session_id)?;
    let raw = signed_request(Method::GET, format!("/v1/sessions/{id}/chat"), vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida do histórico de chat".to_string())
}

#[tauri::command]
pub async fn revoke_session_chat(session_id: String) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    let body = serde_json::to_vec(&serde_json::json!({"capability": "chat"}))
        .map_err(|_| "Falha ao codificar revogação".to_string())?;
    let raw = signed_request(Method::POST, format!("/v1/sessions/{id}/revoke"), body).await?;
    let _: SessionReceipt =
        serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para revogação".to_string())?;
    Ok(())
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
    stop_capture_for(id)?;
    {
        let mut current = ENABLED_HOST_MOUSE
            .lock()
            .map_err(|_| "Falha no controle local do mouse".to_string())?;
        if *current == Some(id) {
            *current = None;
        }
    }
    {
        let mut current = ENABLED_HOST_KEYBOARD
            .lock()
            .map_err(|_| "Falha no controle local do teclado".to_string())?;
        if *current == Some(id) {
            *current = None;
        }
    }
    stop_input_pump(&ACTIVE_MOUSE_PUMP, id);
    stop_input_pump(&ACTIVE_KEYBOARD_PUMP, id);
    let path = format!("/v1/sessions/{id}");
    let raw = signed_request(Method::DELETE, path, vec![]).await?;
    serde_json::from_slice(&raw).map_err(|_| "Resposta inválida para encerramento".to_string())
}

fn host_mouse_is_enabled(id: Uuid) -> Result<bool, String> {
    let enabled = *ENABLED_HOST_MOUSE
        .lock()
        .map_err(|_| "Falha no controle local do mouse".to_string())?;
    let primary = CAPTURE_MONITOR
        .lock()
        .map_err(|_| "Falha ao consultar monitor compartilhado".to_string())?
        .is_none_or(|(_, is_primary)| is_primary);
    Ok(enabled == Some(id) && primary)
}

/// Operator may only submit bounded mouse events signed with its native key.
/// The backend independently checks that Input and Screen were granted.
#[tauri::command]
pub async fn send_remote_mouse(session_id: String, event: MouseEvent) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    event
        .validate()
        .map_err(|_| "Evento de mouse inválido".to_string())?;
    let body = serde_json::to_vec(&event)
        .map_err(|_| "Falha ao codificar o evento do mouse".to_string())?;
    signed_request(Method::POST, format!("/v1/sessions/{id}/input"), body).await?;
    Ok(())
}

/// The host accepts this input capability in the foreground dialog before
/// capture starts. Drain queued events on activation so old clicks cannot
/// execute after the host changes its mind.
#[tauri::command]
pub async fn configure_host_mouse(
    app: tauri::AppHandle,
    session_id: String,
    enabled: bool,
) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    if !enabled {
        let mut current = ENABLED_HOST_MOUSE
            .lock()
            .map_err(|_| "Falha no controle local do mouse".to_string())?;
        if *current == Some(id) {
            *current = None;
        }
        stop_input_pump(&ACTIVE_MOUSE_PUMP, id);
        return Ok(());
    }
    if !native_input_available() {
        return Err("Controle remoto no Wayland requer uma sessão autorizada do portal RemoteDesktop; use uma sessão X11 nesta versão".to_string());
    }
    let authorizations = list_active_authorizations().await?;
    if !authorizations.iter().any(|item| {
        item.session_id == id
            && item.granted.contains(&Capability::Input)
            && item.granted.contains(&Capability::Screen)
    }) {
        return Err("O host não concedeu permissão para controle do mouse".to_string());
    }
    tauri::async_runtime::spawn_blocking(input_backend_ready)
        .await
        .map_err(|_| "Falha ao verificar a entrada nativa".to_string())??;
    // Drain a prior input batch before this user action activates injection.
    let path = format!("/v1/sessions/{id}/input");
    let raw = signed_request(Method::GET, path, vec![]).await?;
    let _: Vec<MouseEvent> = serde_json::from_slice(&raw)
        .map_err(|_| "Resposta inválida do canal de entrada".to_string())?;
    let mut enabled = ENABLED_HOST_MOUSE
        .lock()
        .map_err(|_| "Falha no controle local do mouse".to_string())?;
    if CAPTURE_MONITOR
        .lock()
        .map_err(|_| "Falha ao consultar monitor compartilhado".to_string())?
        .is_some_and(|(_, is_primary)| !is_primary)
    {
        return Err("Mouse remoto só está disponível no monitor principal".into());
    }
    *enabled = Some(id);
    drop(enabled);
    let running = Arc::new(AtomicBool::new(true));
    {
        let mut pump = ACTIVE_MOUSE_PUMP
            .lock()
            .map_err(|_| "Falha ao iniciar o receptor de mouse".to_string())?;
        if let Some((_, previous)) = pump.as_ref() {
            previous.store(false, Ordering::Release);
        }
        *pump = Some((id, running.clone()));
    }
    let worker_running = running.clone();
    let spawned = std::thread::Builder::new()
        .name("seven-mouse-input".into())
        .spawn(move || {
            while worker_running.load(Ordering::Acquire) {
                if !host_mouse_is_enabled(id).unwrap_or(false) {
                    break;
                }
                if let Err(message) =
                    tauri::async_runtime::block_on(poll_host_mouse(id.to_string()))
                {
                    if worker_running.load(Ordering::Acquire) {
                        if let Ok(mut current) = ENABLED_HOST_MOUSE.lock() {
                            if *current == Some(id) {
                                *current = None;
                            }
                        }
                        let _ = app.emit(
                            "seven:host-input-stopped",
                            HostInputStopped {
                                session_id: id,
                                kind: "mouse",
                                message,
                            },
                        );
                    }
                    break;
                }
                std::thread::sleep(Duration::from_millis(180));
            }
            if let Ok(mut pump) = ACTIVE_MOUSE_PUMP.lock() {
                if pump.as_ref().is_some_and(|(session, flag)| {
                    *session == id && Arc::ptr_eq(flag, &worker_running)
                }) {
                    *pump = None;
                }
            }
        });
    if spawned.is_err() {
        stop_input_pump(&ACTIVE_MOUSE_PUMP, id);
        *ENABLED_HOST_MOUSE
            .lock()
            .map_err(|_| "Falha no controle local do mouse".to_string())? = None;
        return Err("Não foi possível iniciar o receptor de mouse".into());
    }
    Ok(())
}

fn normalized_pointer(value: u16, dimension: i32) -> i32 {
    let limit = dimension.saturating_sub(1).max(0) as i64;
    ((i64::from(value) * limit) / 10_000) as i32
}

pub(crate) fn native_input_available() -> bool {
    // X11 injection cannot control Wayland apps. The Wayland RemoteDesktop
    // portal needs its own consented input session before this can be enabled.
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_SESSION_TYPE")
            .map(|kind| kind.eq_ignore_ascii_case("x11"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

fn input_backend_ready() -> Result<(), String> {
    enigo::Enigo::new(&enigo::Settings::default())
        .map(|_| ())
        .map_err(|_| "O sistema não liberou o controle remoto; verifique Acessibilidade no macOS ou a sessão X11 no Linux".to_string())
}

fn apply_mouse_batch(id: Uuid, events: Vec<MouseEvent>) -> Result<usize, String> {
    use enigo::{Axis, Button, Coordinate, Direction, Enigo, Mouse, Settings};
    let mut backend = Enigo::new(&Settings::default()).map_err(|_| {
        "Não foi possível inicializar a entrada nativa; verifique as permissões do sistema"
            .to_string()
    })?;
    let (width, height) = backend
        .main_display()
        .map_err(|_| "Não foi possível identificar o monitor principal".to_string())?;
    if width < 2 || height < 2 {
        return Err("Resolução inválida do monitor principal".to_string());
    }
    let mut applied = 0;
    for event in events {
        // Hold both local gates while applying one event, so a monitor switch
        // or stop action cannot race with a click already dequeued.
        let enabled = ENABLED_HOST_MOUSE
            .lock()
            .map_err(|_| "Falha no controle local do mouse".to_string())?;
        let selected = CAPTURE_MONITOR
            .lock()
            .map_err(|_| "Falha ao consultar monitor compartilhado".to_string())?;
        if *enabled != Some(id) || selected.is_some_and(|(_, is_primary)| !is_primary) {
            break;
        }
        event
            .validate()
            .map_err(|_| "Evento de entrada fora dos limites".to_string())?;
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
            .map_err(|_| "O sistema recusou o movimento do cursor".to_string())?;
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
                    .map_err(|_| "O sistema recusou o clique".to_string())?;
            }
            MouseEvent::Scroll { steps, .. } => {
                backend
                    .scroll(i32::from(steps), Axis::Vertical)
                    .map_err(|_| "O sistema recusou a rolagem".to_string())?;
            }
        }
        applied += 1;
    }
    Ok(applied)
}

/// The host retrieves and applies events only while the local user has
/// explicitly enabled the mouse path. No UAC or elevated desktop.
#[tauri::command]
pub async fn poll_host_mouse(session_id: String) -> Result<usize, String> {
    let id = parse_session_id(&session_id)?;
    if !host_mouse_is_enabled(id)? {
        return Err("Controle do mouse desativado no host".to_string());
    }
    let path = format!("/v1/sessions/{id}/input");
    let raw = signed_request(Method::GET, path, vec![]).await?;
    let events: Vec<MouseEvent> =
        serde_json::from_slice(&raw).map_err(|_| "Lote de entrada inválido".to_string())?;
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

fn host_keyboard_is_enabled(id: Uuid) -> Result<bool, String> {
    Ok(*ENABLED_HOST_KEYBOARD
        .lock()
        .map_err(|_| "Falha no controle local do teclado".to_string())?
        == Some(id))
}

/// The operator sends one printable character or one unmodified special key.
/// The server independently checks the separate Keyboard grant.
#[tauri::command]
pub async fn send_remote_keyboard(session_id: String, event: KeyboardEvent) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    event
        .validate()
        .map_err(|_| "Evento de teclado inválido".to_string())?;
    let body = serde_json::to_vec(&event)
        .map_err(|_| "Falha ao codificar o evento de teclado".to_string())?;
    signed_request(Method::POST, format!("/v1/sessions/{id}/keyboard"), body).await?;
    Ok(())
}

/// The host grants keyboard access in the consent dialog. Every activation
/// drains old queued keys before starting the native receiver.
#[tauri::command]
pub async fn configure_host_keyboard(
    app: tauri::AppHandle,
    session_id: String,
    enabled: bool,
) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    if !enabled {
        let mut current = ENABLED_HOST_KEYBOARD
            .lock()
            .map_err(|_| "Falha no controle local do teclado".to_string())?;
        if *current == Some(id) {
            *current = None;
        }
        stop_input_pump(&ACTIVE_KEYBOARD_PUMP, id);
        return Ok(());
    }
    if !native_input_available() {
        return Err("Controle remoto no Wayland requer uma sessão autorizada do portal RemoteDesktop; use uma sessão X11 nesta versão".to_string());
    }
    let authorized = list_active_authorizations().await?.iter().any(|item| {
        item.session_id == id
            && item.granted.contains(&Capability::Keyboard)
            && item.granted.contains(&Capability::Screen)
    });
    if !authorized {
        return Err("O host não concedeu permissão para controle do teclado".to_string());
    }
    tauri::async_runtime::spawn_blocking(input_backend_ready)
        .await
        .map_err(|_| "Falha ao verificar a entrada nativa".to_string())??;
    let raw = signed_request(Method::GET, format!("/v1/sessions/{id}/keyboard"), vec![]).await?;
    let _: Vec<KeyboardEvent> = serde_json::from_slice(&raw)
        .map_err(|_| "Resposta inválida do canal de teclado".to_string())?;
    *ENABLED_HOST_KEYBOARD
        .lock()
        .map_err(|_| "Falha no controle local do teclado".to_string())? = Some(id);
    let running = Arc::new(AtomicBool::new(true));
    {
        let mut pump = ACTIVE_KEYBOARD_PUMP
            .lock()
            .map_err(|_| "Falha ao iniciar o receptor de teclado".to_string())?;
        if let Some((_, previous)) = pump.as_ref() {
            previous.store(false, Ordering::Release);
        }
        *pump = Some((id, running.clone()));
    }
    let worker_running = running.clone();
    let spawned = std::thread::Builder::new()
        .name("seven-keyboard-input".into())
        .spawn(move || {
            while worker_running.load(Ordering::Acquire) {
                if !host_keyboard_is_enabled(id).unwrap_or(false) {
                    break;
                }
                if let Err(message) =
                    tauri::async_runtime::block_on(poll_host_keyboard(id.to_string()))
                {
                    if worker_running.load(Ordering::Acquire) {
                        if let Ok(mut current) = ENABLED_HOST_KEYBOARD.lock() {
                            if *current == Some(id) {
                                *current = None;
                            }
                        }
                        let _ = app.emit(
                            "seven:host-input-stopped",
                            HostInputStopped {
                                session_id: id,
                                kind: "keyboard",
                                message,
                            },
                        );
                    }
                    break;
                }
                std::thread::sleep(Duration::from_millis(180));
            }
            if let Ok(mut pump) = ACTIVE_KEYBOARD_PUMP.lock() {
                if pump.as_ref().is_some_and(|(session, flag)| {
                    *session == id && Arc::ptr_eq(flag, &worker_running)
                }) {
                    *pump = None;
                }
            }
        });
    if spawned.is_err() {
        stop_input_pump(&ACTIVE_KEYBOARD_PUMP, id);
        *ENABLED_HOST_KEYBOARD
            .lock()
            .map_err(|_| "Falha no controle local do teclado".to_string())? = None;
        return Err("Não foi possível iniciar o receptor de teclado".into());
    }
    Ok(())
}

fn apply_keyboard_batch(id: Uuid, events: Vec<KeyboardEvent>) -> Result<usize, String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut backend = Enigo::new(&Settings::default()).map_err(|_| {
        "Não foi possível inicializar o teclado nativo; verifique as permissões do sistema"
            .to_string()
    })?;
    let mut applied = 0;
    for event in events {
        if !host_keyboard_is_enabled(id)? {
            break;
        }
        event
            .validate()
            .map_err(|_| "Evento de teclado fora dos limites".to_string())?;
        match event {
            KeyboardEvent::Character { value } => backend
                .text(&value.to_string())
                .map_err(|_| "O sistema recusou o caractere remoto".to_string())?,
            KeyboardEvent::Special { key } => {
                let native = match key {
                    SpecialKey::Backspace => Key::Backspace,
                    SpecialKey::Delete => Key::Delete,
                    SpecialKey::Enter => Key::Return,
                    SpecialKey::Tab => Key::Tab,
                    SpecialKey::Escape => Key::Escape,
                    SpecialKey::ArrowLeft => Key::LeftArrow,
                    SpecialKey::ArrowRight => Key::RightArrow,
                    SpecialKey::ArrowUp => Key::UpArrow,
                    SpecialKey::ArrowDown => Key::DownArrow,
                };
                backend
                    .key(native, Direction::Click)
                    .map_err(|_| "O sistema recusou a tecla remota".to_string())?;
            }
        }
        applied += 1;
    }
    Ok(applied)
}

/// Host polling stops when the app closes, loses a grant or screen delivery
/// pauses. Minimized windows continue receiving input.
#[tauri::command]
pub async fn poll_host_keyboard(session_id: String) -> Result<usize, String> {
    let id = parse_session_id(&session_id)?;
    if !host_keyboard_is_enabled(id)? {
        return Err("Controle do teclado desativado no host".to_string());
    }
    let raw = signed_request(Method::GET, format!("/v1/sessions/{id}/keyboard"), vec![]).await?;
    let events: Vec<KeyboardEvent> =
        serde_json::from_slice(&raw).map_err(|_| "Lote de teclado inválido".to_string())?;
    if events.len() > 32 || events.iter().any(|event| event.validate().is_err()) {
        return Err("Lote de teclado excedeu os limites permitidos".to_string());
    }
    if events.is_empty() || !host_keyboard_is_enabled(id)? {
        return Ok(0);
    }
    let authorized = list_active_authorizations().await?.iter().any(|item| {
        item.session_id == id
            && item.granted.contains(&Capability::Keyboard)
            && item.granted.contains(&Capability::Screen)
    });
    if !authorized {
        return Err("Permissão de teclado revogada pelo host".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || apply_keyboard_batch(id, events))
        .await
        .map_err(|_| "Falha na thread nativa de teclado".to_string())?
}

/// Host-only on/off switch for the frame channel; changing to false clears
/// server-side pixels and rejects any late upload from an older capture.
#[tauri::command]
pub async fn set_screen_delivery(
    app: tauri::AppHandle,
    session_id: String,
    enabled: bool,
) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    if !enabled {
        stop_capture_for(id)?;
        configure_host_mouse(app.clone(), session_id.clone(), false).await?;
        configure_host_keyboard(app, session_id.clone(), false).await?;
    }
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

/// Capture from the monitor chosen by the host. The caller checks the session
/// grant before invoking this function.
fn capture_png() -> Result<Vec<u8>, String> {
    use image::{DynamicImage, ImageFormat, imageops::FilterType};
    use std::io::Cursor;

    let selected_at_start = *CAPTURE_MONITOR
        .lock()
        .map_err(|_| "Falha ao consultar monitor compartilhado".to_string())?;
    let monitors = xcap::Monitor::all()
        .map_err(|_| "Não foi possível listar os monitores deste dispositivo".to_string())?;
    let monitor = match selected_at_start {
        Some((id, was_primary)) => {
            let selected = monitors
                .iter()
                .find(|monitor| monitor.id().ok() == Some(id))
                .ok_or_else(|| "Monitor selecionado não está mais disponível".to_string())?;
            if selected.is_primary().unwrap_or(false) != was_primary {
                return Err("A configuração dos monitores mudou; selecione novamente".into());
            }
            selected
        }
        None => monitors
            .iter()
            .find(|monitor| monitor.is_primary().unwrap_or(false))
            .or_else(|| monitors.first())
            .ok_or_else(|| "Monitor principal indisponível; selecione um monitor".to_string())?,
    };
    let screenshot = monitor.capture_image().map_err(|_| {
        "O sistema não autorizou a captura da tela; verifique as permissões".to_string()
    })?;

    let source = DynamicImage::ImageRgba8(screenshot);
    // Keep first-stage frame transfers strictly bounded, without storing pixels.
    // Progressive downscaling also ensures high-resolution desktops don't silently
    // cause an unbounded upload or memory spike.
    for width in [1_600_u32, 1_280, 1_024, 800, 640, 480, 320] {
        let scaled = source.resize(
            width.min(source.width()),
            (width * 2).min(source.height()),
            FilterType::Triangle,
        );
        let mut cursor = Cursor::new(Vec::new());
        scaled
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|_| "Falha ao codificar o quadro da tela".to_string())?;
        let encoded = cursor.into_inner();
        if encoded.len() <= MAX_FRAME_BYTES {
            if *CAPTURE_MONITOR
                .lock()
                .map_err(|_| "Falha ao consultar monitor compartilhado".to_string())?
                != selected_at_start
            {
                return Err("Monitor alterado durante a captura; tente novamente".into());
            }
            return Ok(encoded);
        }
    }
    Err("O quadro excedeu o limite de transferência permitido".to_string())
}

fn encode_recorded_frame(frame: xcap::Frame) -> Result<Vec<u8>, String> {
    use image::{DynamicImage, ImageFormat, RgbaImage, imageops::FilterType};
    use std::io::Cursor;

    if !(1..=8_192).contains(&frame.width) || !(1..=8_192).contains(&frame.height) {
        return Err("Resolução da captura contínua inválida".into());
    }
    let rgba = RgbaImage::from_raw(frame.width, frame.height, frame.raw)
        .ok_or_else(|| "Pixels da captura contínua inválidos".to_string())?;
    let source = DynamicImage::ImageRgba8(rgba);
    for width in [1_280_u32, 960, 800, 640, 480] {
        let scaled = source.resize(
            width.min(source.width()),
            (width * 2).min(source.height()),
            FilterType::Triangle,
        );
        let mut cursor = Cursor::new(Vec::new());
        scaled
            .write_to(&mut cursor, ImageFormat::Jpeg)
            .map_err(|_| "Falha ao codificar a prévia JPEG".to_string())?;
        let encoded = cursor.into_inner();
        if encoded.len() <= MAX_FRAME_BYTES {
            return Ok(encoded);
        }
    }
    Err("Quadro JPEG excedeu o limite permitido".into())
}

fn run_continuous_capture(
    id: Uuid,
    selected: Option<(u32, bool)>,
    running: &AtomicBool,
    ready: std::sync::mpsc::SyncSender<Result<(), String>>,
) -> Result<(), String> {
    use std::sync::mpsc::RecvTimeoutError;

    let prepared = (|| {
        let monitors = xcap::Monitor::all()
            .map_err(|_| "Não foi possível listar os monitores para prévia".to_string())?;
        let monitor = match selected {
            Some((monitor_id, was_primary)) => monitors
                .into_iter()
                .find(|item| {
                    item.id().ok() == Some(monitor_id)
                        && item.is_primary().ok() == Some(was_primary)
                })
                .ok_or_else(|| "Monitor escolhido mudou; selecione novamente".to_string())?,
            None => {
                let primary_index = monitors
                    .iter()
                    .position(|item| item.is_primary().unwrap_or(false))
                    .unwrap_or(0);
                monitors
                    .into_iter()
                    .nth(primary_index)
                    .ok_or_else(|| "Nenhum monitor disponível para captura".to_string())?
            }
        };
        let (recorder, frames) = monitor.video_recorder().map_err(|_| {
            "Não foi possível criar a captura contínua; verifique as permissões da tela".to_string()
        })?;
        recorder
            .start()
            .map_err(|_| "O sistema recusou iniciar a captura contínua".to_string())?;
        Ok::<_, String>((recorder, frames))
    })();
    let (recorder, frames) = match prepared {
        Ok(prepared) => {
            let _ = ready.send(Ok(()));
            prepared
        }
        Err(error) => {
            let _ = ready.send(Err(error.clone()));
            return Err(error);
        }
    };

    const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 15);
    let mut last_sent = Instant::now() - FRAME_INTERVAL;
    let path = format!("/v1/sessions/{id}/frame");
    let result = (|| -> Result<(), String> {
        while running.load(Ordering::Acquire) {
            let mut frame = match frames.recv_timeout(Duration::from_millis(250)) {
                Ok(frame) => frame,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("Captura contínua foi interrompida pelo sistema".into());
                }
            };
            if last_sent.elapsed() < FRAME_INTERVAL {
                continue;
            }
            // A slow encode or network request must not make the viewer watch
            // a queue of stale frames after the connection recovers.
            for _ in 0..120 {
                match frames.try_recv() {
                    Ok(newer) => frame = newer,
                    Err(_) => break,
                }
            }
            if *CAPTURE_MONITOR
                .lock()
                .map_err(|_| "Falha ao consultar monitor compartilhado".to_string())?
                != selected
            {
                return Err("Monitor alterado durante a prévia".into());
            }
            let frame_started = Instant::now();
            let encoded = encode_recorded_frame(frame)?;
            if !running.load(Ordering::Acquire) {
                break;
            }
            tauri::async_runtime::block_on(signed_request(Method::POST, path.clone(), encoded))?;
            last_sent = frame_started;
        }
        Ok(())
    })();
    drop(frames);
    let _ = recorder.stop();
    result
}

/// Starts one consented native capture session. The recorder
/// owns a frame pool; JPEG encoding and network calls are bounded by one
/// worker and never run concurrently for the same host session.
#[tauri::command]
pub async fn start_host_stream_capture(
    app: tauri::AppHandle,
    session_id: String,
) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    {
        let authorized = list_active_authorizations()
            .await?
            .iter()
            .any(|item| item.session_id == id && item.granted.contains(&Capability::Screen));
        if !authorized {
            return Err("O host não autorizou esta sessão de tela".into());
        }
        let selected = *CAPTURE_MONITOR
            .lock()
            .map_err(|_| "Falha ao consultar monitor compartilhado".to_string())?;
        let running = Arc::new(AtomicBool::new(true));
        {
            let mut active = ACTIVE_CAPTURE
                .lock()
                .map_err(|_| "Falha no controle local da captura".to_string())?;
            if active.is_some() {
                return Err(
                    "A captura anterior ainda está ativa ou terminando; aguarde um instante".into(),
                );
            }
            *active = Some((id, running.clone()));
        }
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let worker_running = running.clone();
        let spawn_result = std::thread::Builder::new()
            .name("seven-screen-capture".into())
            .spawn(move || {
                let result = run_continuous_capture(id, selected, &worker_running, ready_tx);
                let was_running = worker_running.swap(false, Ordering::AcqRel);
                if was_running {
                    if let Err(message) = result {
                        let _ = app.emit(
                            "seven:stream-stopped",
                            CaptureStopped {
                                session_id: id,
                                message,
                            },
                        );
                    }
                }
                if let Ok(mut active) = ACTIVE_CAPTURE.lock() {
                    if active.as_ref().is_some_and(|(current, flag)| {
                        *current == id && Arc::ptr_eq(flag, &worker_running)
                    }) {
                        *active = None;
                    }
                }
            });
        if spawn_result.is_err() {
            *ACTIVE_CAPTURE
                .lock()
                .map_err(|_| "Falha no controle local da captura".to_string())? = None;
            return Err("Não foi possível iniciar a thread de captura".into());
        }
        let permission_prompt_expected = cfg!(target_os = "macos")
            || (cfg!(target_os = "linux")
                && std::env::var("XDG_SESSION_TYPE")
                    .map(|kind| kind.eq_ignore_ascii_case("wayland"))
                    .unwrap_or(false));
        let ready_timeout = if permission_prompt_expected {
            Duration::from_secs(90)
        } else {
            Duration::from_secs(6)
        };
        let ready_result =
            tauri::async_runtime::spawn_blocking(move || ready_rx.recv_timeout(ready_timeout))
                .await
                .map_err(|_| "Falha ao aguardar captura contínua".to_string())?;
        match ready_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => {
                stop_capture_for(id)?;
                Err(error)
            }
            Err(_) => {
                stop_capture_for(id)?;
                Err("Captura contínua não iniciou a tempo".into())
            }
        }
    }
}

#[tauri::command]
pub fn stop_host_stream_capture(session_id: String) -> Result<(), String> {
    stop_capture_for(parse_session_id(&session_id)?)
}

/// This command is an explicit action from the local host, not a background
/// stream. The server rechecks host identity and Screen permission upon upload.
#[tauri::command]
pub async fn publish_manual_screen_frame(session_id: String) -> Result<(), String> {
    let id = parse_session_id(&session_id)?;
    let authorizations = list_active_authorizations().await?;
    if !authorizations
        .iter()
        .any(|item| item.session_id == id && item.granted.contains(&Capability::Screen))
    {
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
    let frame = signed_request(Method::GET, path, vec![]).await?;
    let mime = if frame.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if frame.starts_with(b"\xff\xd8") && frame.ends_with(b"\xff\xd9") {
        "image/jpeg"
    } else {
        return Err("O servidor retornou um quadro inválido".to_string());
    };
    if frame.len() > MAX_FRAME_BYTES {
        return Err("O servidor retornou um quadro inválido".to_string());
    }
    Ok(format!("data:{mime};base64,{}", BASE64.encode(frame)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_encoder_accepts_rgba_capture_pixels() {
        let jpeg =
            encode_recorded_frame(xcap::Frame::new(2, 2, [10, 20, 30, 255].repeat(4))).unwrap();
        assert!(jpeg.starts_with(b"\xff\xd8") && jpeg.ends_with(b"\xff\xd9"));
        assert!(encode_recorded_frame(xcap::Frame::new(2, 2, vec![0; 3])).is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    #[ignore = "requires an interactive Windows desktop; pixels stay in memory"]
    fn interactive_screen_capture_uses_native_recorder() {
        let monitors = xcap::Monitor::all().expect("desktop monitor available");
        let monitor = monitors
            .iter()
            .find(|item| item.is_primary().unwrap_or(false))
            .or_else(|| monitors.first())
            .expect("at least one desktop monitor");
        let png = capture_png().expect("manual native frame");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        let (recorder, frames) = monitor.video_recorder().expect("native recorder");
        recorder.start().expect("native recorder starts");
        let frame = frames
            .recv_timeout(Duration::from_secs(5))
            .expect("native frame arrives");
        assert!(frame.width > 0 && frame.height > 0);
        assert_eq!(
            frame.raw.len(),
            frame.width as usize * frame.height as usize * 4
        );
        drop(frames);
        recorder.stop().expect("native recorder stops");
    }

    #[test]
    fn invitation_only_requests_screen_and_has_short_expiry() {
        let operator = Uuid::new_v4();
        let host = Uuid::new_v4();
        let request = invitation(operator, host, 1_000, false, false, false).unwrap();
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
        let view = invitation(source, dest, 100, false, false, false).unwrap();
        let control = invitation(source, dest, 100, true, false, false).unwrap();
        let keyboard = invitation(source, dest, 100, false, true, false).unwrap();
        let chat = invitation(source, dest, 100, false, false, true).unwrap();
        assert_eq!(view.requested, BTreeSet::from([Capability::Screen]));
        assert_eq!(
            control.requested,
            BTreeSet::from([Capability::Screen, Capability::Input])
        );
        assert_eq!(
            keyboard.requested,
            BTreeSet::from([Capability::Screen, Capability::Keyboard])
        );
        assert_eq!(
            chat.requested,
            BTreeSet::from([Capability::Screen, Capability::Chat])
        );
        assert_eq!(normalized_pointer(0, 1920), 0);
        assert_eq!(normalized_pointer(10_000, 1920), 1919);
        assert_eq!(normalized_pointer(5_000, 1080), 539);
        assert_eq!(normalized_pointer(8_000, 0), 0);
    }

    #[test]
    fn self_invitation_and_invalid_session_ids_fail_closed() {
        let id = Uuid::new_v4();
        assert!(invitation(id, id, 1_000, true, true, true).is_err());
        assert!(parse_session_id("not-a-uuid").is_err());
        assert_eq!(parse_session_id(&id.to_string()), Ok(id));
    }
}

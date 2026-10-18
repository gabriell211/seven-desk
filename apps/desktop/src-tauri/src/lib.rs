#![forbid(unsafe_code)]

mod autostart;
mod server_config;
mod session_api;
mod rustdesk_transport;

use server_config::configured_server_url;
use tauri::Manager;

use serde::Serialize;
use seven_identity::{Identity, SystemKeyStore};
use std::{
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

static INIT_IDENTITY: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
struct LocalStatus {
    app_name: &'static str,
    app_version: &'static str,
    operating_system: &'static str,
    architecture: &'static str,
    device_id: Option<String>,
    host_service_available: bool,
    native_input_available: bool,
}

#[tauri::command]
async fn local_status() -> LocalStatus {
    let device_id = tauri::async_runtime::spawn_blocking(|| {
        Identity::load(&SystemKeyStore)
            .ok()
            .flatten()
            .map(|identity| identity.id().0.to_string())
    })
    .await
    .ok()
    .flatten();

    LocalStatus {
        app_name: "Seven Desk",
        app_version: env!("CARGO_PKG_VERSION"),
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        device_id,
        // Remote control is never enabled by merely creating an identity.
        host_service_available: true,
        native_input_available: session_api::native_input_available(),
    }
}

#[tauri::command]
async fn initialize_identity() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let _guard = INIT_IDENTITY
            .lock()
            .map_err(|_| "Falha ao proteger a inicialização da identidade".to_string())?;
        let identity = Identity::load_or_create(&SystemKeyStore).map_err(|_| {
            "Não foi possível acessar o armazenamento seguro do sistema".to_string()
        })?;
        Ok(identity.id().0.to_string())
    })
    .await
    .map_err(|_| "Falha ao inicializar a identidade".to_string())?
}

/// Submit a signed request using the native identity. The signing key never
/// leaves the Rust process; this developer API only binds to loopback.
async fn send_presence_request(path: &'static str) -> Result<String, String> {
    let base = configured_server_url()?;
    let (device_id, proof) = tauri::async_runtime::spawn_blocking(move || {
        let identity = Identity::load(&SystemKeyStore)
            .map_err(|_| "Cofre de credenciais indisponível".to_string())?
            .ok_or_else(|| "Ative a identidade do dispositivo primeiro".to_string())?;
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Relógio do sistema inválido".to_string())?
            .as_millis() as u64;
        let proof = identity
            .sign("POST", path, b"{}", Uuid::new_v4(), timestamp_ms)
            .map_err(|_| "Falha ao assinar a solicitação".to_string())?;
        Ok::<_, String>((identity.id().0.to_string(), proof))
    })
    .await
    .map_err(|_| "Falha ao preparar a identidade".to_string())??;

    let client = server_config::http_client(Duration::from_secs(5))?;
    let response = client
        .post(format!("{base}{path}"))
        .header("x-seven-key", proof.public_key_hex)
        .header("x-seven-signature", proof.signature_hex)
        .header("x-seven-nonce", proof.nonce.to_string())
        .header("x-seven-timestamp", proof.timestamp_ms.to_string())
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .map_err(|_| "Servidor local indisponível".to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Servidor recusou a solicitação de presença (HTTP {})",
            response.status().as_u16()
        ));
    }
    Ok(device_id)
}

#[tauri::command]
async fn register_local_device() -> Result<String, String> {
    send_presence_request("/v1/devices").await
}

#[tauri::command]
async fn heartbeat_local_device() -> Result<String, String> {
    send_presence_request("/v1/devices/heartbeat").await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            server_config::load_saved_server(app.handle());
            server_config::load_saved_proxy(app.handle());
            let menu = tauri::menu::MenuBuilder::new(app)
                .text("open", "Abrir Seven Desk")
                .text("quit", "Sair do Seven Desk")
                .build()?;
            if let Some(icon) = app.default_window_icon().cloned() {
                tauri::tray::TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("Seven Desk — pronto para pedidos autorizados")
                    .menu(&menu)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .build(app)?;
            }
            if !std::env::args().any(|argument| argument == "--background") {
                if let Some(window) = app.get_webview_window("main") {
                    window.show()?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            local_status,
            initialize_identity,
            register_local_device,
            heartbeat_local_device,
            server_config::backend_status,
            server_config::relay_status,
            rustdesk_transport::probe_rustdesk_server,
            server_config::server_configuration,
            server_config::validate_server_configuration,
            server_config::save_server_configuration,
            server_config::proxy_configuration,
            server_config::validate_proxy_configuration,
            server_config::save_proxy_configuration,
            autostart::autostart_status,
            autostart::set_autostart,
            session_api::create_session_request,
            session_api::list_pending_invitations,
            session_api::list_active_authorizations,
            session_api::list_active_operator_sessions,
            session_api::decide_session,
            session_api::send_session_chat,
            session_api::list_session_chat,
            session_api::check_device_presence,
            session_api::revoke_session_chat,
            session_api::session_status,
            session_api::set_screen_delivery,
            session_api::send_remote_mouse,
            session_api::configure_host_mouse,
            session_api::poll_host_mouse,
            session_api::send_remote_keyboard,
            session_api::configure_host_keyboard,
            session_api::poll_host_keyboard,
            session_api::publish_manual_screen_frame,
            session_api::start_host_stream_capture,
            session_api::stop_host_stream_capture,
            session_api::list_host_monitors,
            session_api::select_host_monitor,
            session_api::fetch_authorized_screen_frame,
            session_api::end_session
        ])
        .run(tauri::generate_context!())
        .expect("unable to start Seven Desk");
}

#![forbid(unsafe_code)]

mod session_api;

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
        host_service_available: false,
    }
}

#[tauri::command]
async fn initialize_identity() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let _guard = INIT_IDENTITY
            .lock()
            .map_err(|_| "Falha ao proteger a inicialização da identidade".to_string())?;
        let identity = Identity::load_or_create(&SystemKeyStore)
            .map_err(|_| "Não foi possível acessar o armazenamento seguro do sistema".to_string())?;
        Ok(identity.id().0.to_string())
    })
    .await
    .map_err(|_| "Falha ao inicializar a identidade".to_string())?
}

/// Submit a signed request using the native identity. The signing key never
/// leaves the Rust process; this developer API only binds to loopback.
async fn send_presence_request(path: &'static str) -> Result<String, String> {
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

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| "Não foi possível inicializar a conexão local".to_string())?;
    let response = client
        .post(format!("http://127.0.0.1:43821{path}"))
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
        .invoke_handler(tauri::generate_handler![
            local_status,
            initialize_identity,
            register_local_device,
            heartbeat_local_device,
            session_api::create_session_request,
            session_api::list_pending_invitations,
            session_api::list_active_authorizations,
            session_api::decide_session,
            session_api::session_status,
            session_api::end_session
        ])
        .run(tauri::generate_context!())
        .expect("unable to start Seven Desk");
}

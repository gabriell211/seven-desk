#![forbid(unsafe_code)]

use serde::Serialize;
use seven_identity::{Identity, SystemKeyStore};
use std::sync::Mutex;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![local_status, initialize_identity])
        .run(tauri::generate_context!())
        .expect("unable to start Seven Desk");
}

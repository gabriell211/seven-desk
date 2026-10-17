#![forbid(unsafe_code)]

use serde::Serialize;

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
fn local_status() -> LocalStatus {
    LocalStatus {
        app_name: "Seven Desk",
        app_version: env!("CARGO_PKG_VERSION"),
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        // Identity and privileged host service cannot be faked.
        device_id: None,
        host_service_available: false,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![local_status])
        .run(tauri::generate_context!())
        .expect("unable to start Seven Desk");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn does_not_claim_uninstalled_service_is_running() {
        let info = local_status();
        assert!(info.device_id.is_none());
        assert!(!info.host_service_available);
    }
}

//! Restricted control-plane configuration for lab and self-hosted testing.
//! No arbitrary plaintext endpoint may receive device identity signatures.
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{fs, sync::RwLock, time::Duration};
use tauri::Manager;

const DEFAULT_SERVER: &str = "http://127.0.0.1:43821";
const MAX_HEALTH_BYTES: usize = 4_096;
const MAX_SERVER_URL_BYTES: usize = 2_048;
const SERVER_FILE: &str = "server-url.txt";
const PROXY_FILE: &str = "proxy.json";
static SAVED_SERVER: RwLock<Option<String>> = RwLock::new(None);
static SAVED_PROXY: RwLock<Option<ProxyConfiguration>> = RwLock::new(None);

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct ProxyConfiguration {
    mode: String,
    url: String,
    environment_override: bool,
}

fn validate_proxy(mode: &str, address: &str) -> Result<ProxyConfiguration, String> {
    let mode = mode.trim();
    if !matches!(mode, "direct" | "system" | "manual") {
        return Err("Modo de proxy inválido".into());
    }
    if mode != "manual" {
        return Ok(ProxyConfiguration {
            mode: mode.into(),
            url: String::new(),
            environment_override: false,
        });
    }
    if address.len() > MAX_SERVER_URL_BYTES {
        return Err("Endereço do proxy longo demais".into());
    }
    let url = Url::parse(address.trim()).map_err(|_| "Endereço do proxy inválido".to_string())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(
            "Use uma URL HTTP(S) de proxy, sem usuário, senha, caminho ou parâmetros".into(),
        );
    }
    Ok(ProxyConfiguration {
        mode: mode.into(),
        url: url.as_str().trim_end_matches('/').into(),
        environment_override: false,
    })
}

fn proxy_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(PROXY_FILE))
        .map_err(|_| "Não foi possível localizar as configurações do aplicativo".to_string())
}

pub fn load_saved_proxy(app: &tauri::AppHandle) {
    let Ok(path) = proxy_path(app) else { return };
    let Ok(metadata) = fs::metadata(&path) else {
        return;
    };
    if metadata.len() > MAX_SERVER_URL_BYTES as u64 + 256 {
        return;
    }
    let Ok(raw) = fs::read(path) else { return };
    let Ok(saved) = serde_json::from_slice::<ProxyConfiguration>(&raw) else {
        return;
    };
    let Ok(validated) = validate_proxy(&saved.mode, &saved.url) else {
        return;
    };
    if let Ok(mut current) = SAVED_PROXY.write() {
        *current = Some(validated);
    }
}

fn configured_proxy() -> Result<ProxyConfiguration, String> {
    if let Ok(address) = std::env::var("SEVEN_DESK_PROXY_URL") {
        let mut configured = validate_proxy("manual", &address)?;
        configured.environment_override = true;
        return Ok(configured);
    }
    Ok(SAVED_PROXY
        .read()
        .map_err(|_| "Falha ao ler o proxy".to_string())?
        .clone()
        .unwrap_or_else(|| ProxyConfiguration {
            mode: "direct".into(),
            ..Default::default()
        }))
}

#[tauri::command]
pub fn proxy_configuration() -> Result<ProxyConfiguration, String> {
    configured_proxy()
}

#[tauri::command]
pub fn validate_proxy_configuration(
    mode: String,
    url: String,
) -> Result<ProxyConfiguration, String> {
    validate_proxy(&mode, &url)
}

#[tauri::command]
pub fn save_proxy_configuration(
    app: tauri::AppHandle,
    mode: String,
    url: String,
) -> Result<ProxyConfiguration, String> {
    if std::env::var_os("SEVEN_DESK_PROXY_URL").is_some() {
        return Err("O proxy está definido pela variável de ambiente desta execução".into());
    }
    let validated = validate_proxy(&mode, &url)?;
    let path = proxy_path(&app)?;
    let directory = path
        .parent()
        .ok_or_else(|| "Pasta de configuração inválida".to_string())?;
    fs::create_dir_all(directory)
        .map_err(|_| "Não foi possível criar a pasta de configuração".to_string())?;
    let encoded = serde_json::to_vec(&validated)
        .map_err(|_| "Não foi possível preparar a configuração do proxy".to_string())?;
    fs::write(path, encoded).map_err(|_| "Não foi possível salvar o proxy".to_string())?;
    *SAVED_PROXY
        .write()
        .map_err(|_| "Falha ao aplicar o proxy".to_string())? = Some(validated.clone());
    Ok(validated)
}

pub fn http_client(timeout: Duration) -> Result<reqwest::Client, String> {
    let base = configured_server_url()?;
    let server = Url::parse(&base).map_err(|_| "Servidor inválido".to_string())?;
    let local = matches!(
        server.host_str(),
        Some("127.0.0.1" | "localhost" | "[::1]" | "::1")
    );
    let proxy = configured_proxy()?;
    let mut builder = reqwest::Client::builder().timeout(timeout);
    if local || proxy.mode == "direct" {
        builder = builder.no_proxy();
    } else if proxy.mode == "manual" {
        builder = builder.proxy(
            reqwest::Proxy::all(&proxy.url)
                .map_err(|_| "Não foi possível configurar o proxy".to_string())?,
        );
    }
    builder
        .build()
        .map_err(|_| "Não foi possível inicializar a conexão de rede".to_string())
}

pub fn validate_server_url(input: &str) -> Result<String, String> {
    if input.len() > MAX_SERVER_URL_BYTES {
        return Err("Endereço do servidor longo demais".into());
    }
    let url = Url::parse(input.trim()).map_err(|_| "Endereço do servidor inválido".to_string())?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(
            "O servidor deve ser uma URL base, sem credenciais, caminho ou parâmetros".into(),
        );
    }
    let host = url
        .host_str()
        .ok_or_else(|| "O servidor precisa ter um nome de host".to_string())?;
    let is_local = matches!(host, "127.0.0.1" | "localhost" | "[::1]" | "::1");
    if url.scheme() != "https" && !(url.scheme() == "http" && is_local) {
        return Err("Conexões fora do loopback exigem HTTPS com certificado válido".into());
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

pub fn configured_server_url() -> Result<String, String> {
    if let Ok(value) = std::env::var("SEVEN_DESK_DEV_SERVER_URL") {
        return validate_server_url(&value);
    }
    let saved = SAVED_SERVER
        .read()
        .map_err(|_| "Falha ao ler a configuração do servidor".to_string())?;
    Ok(saved.clone().unwrap_or_else(|| DEFAULT_SERVER.to_string()))
}

fn config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(SERVER_FILE))
        .map_err(|_| "Não foi possível localizar as configurações do aplicativo".to_string())
}

pub fn load_saved_server(app: &tauri::AppHandle) {
    let Ok(path) = config_path(app) else { return };
    let Ok(metadata) = fs::metadata(&path) else {
        return;
    };
    if metadata.len() > MAX_SERVER_URL_BYTES as u64 {
        return;
    }
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    let Ok(validated) = validate_server_url(&raw) else {
        return;
    };
    if let Ok(mut saved) = SAVED_SERVER.write() {
        *saved = Some(validated);
    }
}

#[derive(Serialize)]
pub struct ServerConfiguration {
    url: String,
    environment_override: bool,
}

#[tauri::command]
pub fn server_configuration() -> Result<ServerConfiguration, String> {
    Ok(ServerConfiguration {
        url: configured_server_url()?,
        environment_override: std::env::var_os("SEVEN_DESK_DEV_SERVER_URL").is_some(),
    })
}

#[tauri::command]
pub fn validate_server_configuration(url: String) -> Result<String, String> {
    validate_server_url(&url)
}

#[tauri::command]
pub fn save_server_configuration(
    app: tauri::AppHandle,
    url: String,
) -> Result<ServerConfiguration, String> {
    if std::env::var_os("SEVEN_DESK_DEV_SERVER_URL").is_some() {
        return Err("O servidor está definido pela variável de ambiente desta execução".into());
    }
    let validated = validate_server_url(&url)?;
    let path = config_path(&app)?;
    let directory = path
        .parent()
        .ok_or_else(|| "Pasta de configuração inválida".to_string())?;
    fs::create_dir_all(directory)
        .map_err(|_| "Não foi possível criar a pasta de configuração".to_string())?;
    fs::write(path, validated.as_bytes())
        .map_err(|_| "Não foi possível salvar o servidor configurado".to_string())?;
    *SAVED_SERVER
        .write()
        .map_err(|_| "Falha ao aplicar a configuração do servidor".to_string())? =
        Some(validated.clone());
    Ok(ServerConfiguration {
        url: validated,
        environment_override: false,
    })
}

#[derive(Serialize, Deserialize)]
pub struct BackendStatus {
    status: String,
    release: String,
    protocol: u16,
    production_ready: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RelayStatus {
    status: String,
    transport: String,
    production_ready: bool,
}

#[tauri::command]
pub async fn relay_status() -> Result<RelayStatus, String> {
    let base = configured_server_url()?;
    let client = http_client(Duration::from_secs(5))?;
    let mut response = client
        .get(format!("{base}/v1/relay/healthz"))
        .send()
        .await
        .map_err(|_| "Relay indisponível".to_string())?;
    if !response.status().is_success() {
        return Err("Relay indisponível".into());
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_HEALTH_BYTES as u64)
    {
        return Err("Resposta do relay excedeu o limite".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Resposta do relay inválida".to_string())?
    {
        if chunk.len() > MAX_HEALTH_BYTES.saturating_sub(bytes.len()) {
            return Err("Resposta do relay excedeu o limite".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let parsed: RelayStatus =
        serde_json::from_slice(&bytes).map_err(|_| "Resposta do relay inválida".to_string())?;
    if parsed.status != "ok" || parsed.transport != "signed-http-relay" || parsed.production_ready {
        return Err("Relay incompatível com este cliente".into());
    }
    Ok(parsed)
}

#[tauri::command]
pub async fn backend_status() -> Result<BackendStatus, String> {
    let base = configured_server_url()?;
    let client = http_client(Duration::from_secs(5))?;
    let mut response = client
        .get(format!("{base}/healthz"))
        .send()
        .await
        .map_err(|_| {
            "Servidor de desenvolvimento indisponível ou certificado inválido".to_string()
        })?;
    if !response.status().is_success() {
        return Err(format!(
            "Falha ao consultar o servidor (HTTP {})",
            response.status().as_u16()
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_HEALTH_BYTES as u64)
    {
        return Err("Resposta de saúde excedeu o limite permitido".to_string());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Erro ao ler resposta de saúde".to_string())?
    {
        if chunk.len() > MAX_HEALTH_BYTES.saturating_sub(bytes.len()) {
            return Err("Resposta de saúde excedeu o limite permitido".to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    let parsed: BackendStatus = serde_json::from_slice(&bytes)
        .map_err(|_| "Resposta do control plane inválida".to_string())?;
    if parsed.status != "ok"
        || parsed.production_ready
        || parsed.protocol != seven_protocol::PROTOCOL_VERSION
    {
        return Err("Servidor incompatível com o modo de desenvolvimento".to_string());
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_local_http_and_remote_https_only() {
        assert_eq!(
            validate_server_url("http://127.0.0.1:43821").unwrap(),
            "http://127.0.0.1:43821"
        );
        assert_eq!(
            validate_server_url("https://desk.example.org/").unwrap(),
            "https://desk.example.org"
        );
        assert!(validate_server_url("http://192.168.0.5:43821").is_err());
        assert!(validate_server_url("http://evil.example.org").is_err());
        assert!(validate_server_url("ftp://evil.example.org").is_err());
        assert!(validate_server_url("https://name:password@desk.example.org").is_err());
        assert!(validate_server_url("https://desk.example.org/api/v1").is_err());
        assert!(validate_server_url("https://desk.example.org/?token=abc").is_err());
        assert!(validate_server_url("https://desk.example.org/#fragment").is_err());
        assert!(validate_server_url(&format!("https://{}", "a".repeat(2_048))).is_err());
    }

    #[test]
    fn proxy_rejects_credentials_and_extra_url_parts() {
        assert_eq!(
            validate_proxy("manual", "http://proxy.example.org:8080")
                .unwrap()
                .url,
            "http://proxy.example.org:8080"
        );
        assert!(validate_proxy("manual", "http://user:secret@proxy.example.org").is_err());
        assert!(validate_proxy("manual", "http://proxy.example.org/path").is_err());
        assert!(validate_proxy("manual", "ftp://proxy.example.org").is_err());
        assert!(validate_proxy("unknown", "").is_err());
        assert!(validate_proxy("direct", "").is_ok());
    }
}

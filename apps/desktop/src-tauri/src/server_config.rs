//! Restricted control-plane configuration for lab and self-hosted testing.
//! No arbitrary plaintext endpoint may receive device identity signatures.
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_SERVER: &str = "http://127.0.0.1:43821";
const MAX_HEALTH_BYTES: usize = 4_096;

pub fn validate_server_url(input: &str) -> Result<String, String> {
    let url = Url::parse(input.trim())
        .map_err(|_| "Endereço do servidor inválido".to_string())?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err("O servidor deve ser uma URL base, sem credenciais, caminho ou parâmetros".into());
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
    let value = std::env::var("SEVEN_DESK_DEV_SERVER_URL")
        .unwrap_or_else(|_| DEFAULT_SERVER.to_string());
    validate_server_url(&value)
}

#[derive(Serialize, Deserialize)]
pub struct BackendStatus {
    status: String,
    release: String,
    protocol: u16,
    production_ready: bool,
}

#[tauri::command]
pub async fn backend_status() -> Result<BackendStatus, String> {
    let base = configured_server_url()?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| "Não foi possível inicializar a conexão TLS".to_string())?;
    let mut response = client
        .get(format!("{base}/healthz"))
        .send()
        .await
        .map_err(|_| "Servidor de desenvolvimento indisponível ou certificado inválido".to_string())?;
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
    if parsed.status != "ok" || parsed.production_ready {
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
    }
}

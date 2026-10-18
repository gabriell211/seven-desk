//! RustDesk OSS rendezvous wire-level diagnostic (NOT a session transport).
//! Only verifies that hbbs responds to a harmless TestNatRequest via TCP.
//! A successful response proves neither server identity nor E2EE readiness.
use serde::Serialize;
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

const HBBS_PORT: u16 = 21116;
const MAX_RESPONSE_BYTES: usize = 4096;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Serialize)]
pub struct RustDeskServerProbe {
    host: String,
    port: u16,
    observed_port: u16,
    protocol: &'static str,
    authenticated: bool,
    sessions_enabled: bool,
}

fn validate_host(input: &str) -> Result<&str, String> {
    let host = input.trim();
    if host.is_empty() || host.len() > 253 {
        return Err("Informe um nome de servidor válido (até 253 caracteres)".into());
    }
    // Only a host, not URL, username, credentials, path or arbitrary port.
    if !host.is_ascii()
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        || host.starts_with('-')
        || host.starts_with('.')
        || host.ends_with('-')
        || host.ends_with('.')
        || host.split('.').any(|label| label.is_empty() || label.len() > 63 ||
            label.starts_with('-') || label.ends_with('-'))
    {
        return Err("Informe apenas um hostname ou IPv4, sem http://, porta ou caminho".into());
    }
    Ok(host)
}

fn read_varint(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..=63).step_by(7) {
        let byte = *data.get(*pos)?;
        *pos += 1;
        if shift == 63 && byte > 1 {
            return None;
        }
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

fn next_field<'a>(data: &'a [u8], pos: &mut usize) -> Option<(u64, u64, Option<&'a [u8]>)> {
    let tag = read_varint(data, pos)?;
    let number = tag >> 3;
    let wire = tag & 7;
    if number == 0 {
        return None;
    }
    match wire {
        0 => Some((number, read_varint(data, pos)?, None)),
        1 => {
            *pos = pos.checked_add(8)?;
            data.get(..*pos)?;
            Some((number, 0, None))
        }
        2 => {
            let len = usize::try_from(read_varint(data, pos)?).ok()?;
            let end = pos.checked_add(len)?;
            let slice = data.get(*pos..end)?;
            *pos = end;
            Some((number, 0, Some(slice)))
        }
        5 => {
            *pos = pos.checked_add(4)?;
            data.get(..*pos)?;
            Some((number, 0, None))
        }
        _ => None,
    }
}

fn parse_nat_response(payload: &[u8]) -> Option<u16> {
    let mut pos = 0;
    while pos < payload.len() {
        let (field, _, value) = next_field(payload, &mut pos)?;
        if field == 21 {
            let nested = value?;
            let mut nested_pos = 0;
            while nested_pos < nested.len() {
                let (nested_field, raw, _) = next_field(nested, &mut nested_pos)?;
                if nested_field == 1 {
                    return u16::try_from(raw).ok().filter(|port| *port > 0);
                }
            }
        }
    }
    None
}

fn read_framed_response(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut first = [0; 1];
    stream.read_exact(&mut first)
        .map_err(|_| "hbbs não respondeu ao diagnóstico Protobuf".to_string())?;
    let header_len = usize::from((first[0] & 3) + 1);
    let mut header = [0u8; 4];
    header[0] = first[0];
    if header_len > 1 {
        stream.read_exact(&mut header[1..header_len])
            .map_err(|_| "Cabeçalho de resposta truncado".to_string())?;
    }
    let len = (u32::from_le_bytes(header) >> 2) as usize;
    if len == 0 || len > MAX_RESPONSE_BYTES {
        return Err("Tamanho de resposta hbbs inválido".into());
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload)
        .map_err(|_| "Resposta hbbs incompleta".to_string())?;
    Ok(payload)
}

fn probe_hbbs(host: &str) -> Result<RustDeskServerProbe, String> {
    let addresses = (host, HBBS_PORT)
        .to_socket_addrs()
        .map_err(|_| "Não foi possível resolver o DNS do servidor hbbs".to_string())?;
    let mut had_address = false;
    let mut last_error = "O servidor não respondeu no TCP 21116".to_string();
    for addr in addresses.take(4) {
        had_address = true;
        let Ok(mut stream) = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) else {
            continue;
        };
        stream.set_read_timeout(Some(CONNECT_TIMEOUT)).ok();
        stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();
        // Framed RustDesk: 1-byte (length << 2); protobuf field 20 (TestNatRequest), empty payload.
        if stream.write_all(&[0x0c, 0xa2, 0x01, 0x00]).is_err() {
            last_error = "Não foi possível enviar o TestNatRequest ao hbbs".into();
            continue;
        }
        match read_framed_response(&mut stream) {
            Ok(data) => {
                let observed_port = parse_nat_response(&data)
                    .ok_or_else(|| "Servidor retornou um protocolo incompatível".to_string())?;
                return Ok(RustDeskServerProbe {
                    host: host.into(),
                    port: HBBS_PORT,
                    observed_port,
                    protocol: "rustdesk-rendezvous-protobuf",
                    authenticated: false,
                    sessions_enabled: false,
                });
            }
            Err(err) => last_error = err,
        }
    }
    if !had_address {
        return Err("Não foram encontrados endereços válidos para o servidor hbbs".into());
    }
    Err(last_error)
}

#[tauri::command]
pub async fn probe_rustdesk_server(host: String) -> Result<RustDeskServerProbe, String> {
    let validated = validate_host(&host)?.to_string();
    tauri::async_runtime::spawn_blocking(move || probe_hbbs(&validated))
        .await
        .map_err(|_| "Falha ao concluir o diagnóstico de rede".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_validation_rejects_urls_credentials_and_ports() {
        for input in ["", "https://desk.exemplo.com", "desk.exemplo.com:21116", "a@b", "ab/c", "-a", "a..b", "a.", "xn--.x"] {
            assert!(validate_host(input).is_err(), "should reject: {input}");
        }
        assert_eq!(validate_host(" desk.exemplo.com "), Ok("desk.exemplo.com"));
        assert_eq!(validate_host("127.0.0.1"), Ok("127.0.0.1"));
    }

    #[test]
    fn protocol_uses_real_rustdesk_test_nat_response_field_21() {
        // TestNatResponse { port: 21116 }, wrapped in RendezvousMessage test_nat_response = 21.
        let data = [0xaa, 0x01, 0x04, 0x08, 0xfc, 0xa4, 0x01];
        assert_eq!(parse_nat_response(&data), Some(21116));
        assert_eq!(parse_nat_response(&[0xa2, 0x01, 0x00]), None);
        assert_eq!(parse_nat_response(&[0xaa, 0x01, 0x04, 0x08]), None);
        assert_eq!(parse_nat_response(&[0xaa, 0x01, 0x02, 0x08, 0x00]), None);
        assert_eq!(parse_nat_response(&[0x80]), None);
    }

    #[test]
    fn framed_decoder_rejects_oversize_and_truncation() {
        use std::net::TcpListener;
        use std::thread;
        fn run(reply: Vec<u8>) -> Result<Vec<u8>, String> {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let sender = thread::spawn(move || {
                let (mut conn, _) = listener.accept().unwrap();
                conn.write_all(&reply).unwrap();
            });
            let mut client = TcpStream::connect(address).unwrap();
            client.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
            let result = read_framed_response(&mut client);
            sender.join().unwrap();
            result
        }
        assert_eq!(run(vec![0x1c, 1, 2, 3, 4, 5, 6, 7]).unwrap(), vec![1, 2, 3, 4, 5, 6, 7]);
        assert!(run(vec![0x01, 0xff]).is_err()); // header says 16k+
        assert!(run(vec![0x0c, 0xaa]).is_err()); // expected three bytes
    }
}

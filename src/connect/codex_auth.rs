use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use crate::providers::http_agent;

const ISSUER: &str = "https://auth.openai.com";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const POLL_TIMEOUT_SECS: u64 = 15 * 60;

#[derive(Clone, Debug)]
pub struct CodexTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub source: String,
}

pub fn ensure_codex_session() -> Result<CodexTokens, String> {
    if let Some(tokens) = load_saved_tokens() {
        return Ok(tokens);
    }
    if let Some(tokens) = import_cli_tokens() {
        println!("Using existing Codex CLI credentials from ~/.codex/auth.json");
        return Ok(tokens);
    }
    device_code_login()
}

pub fn import_cli_tokens() -> Option<CodexTokens> {
    let path = codex_home()?.join("auth.json");
    let text = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let access = value["tokens"]["access_token"].as_str()?.to_owned();
    let refresh = value["tokens"]["refresh_token"]
        .as_str()
        .map(str::to_owned);
    if token_expired(&access, 0) {
        return None;
    }
    Some(CodexTokens {
        access_token: access,
        refresh_token: refresh,
        source: "cli_import".to_owned(),
    })
}

fn load_saved_tokens() -> Option<CodexTokens> {
    let entry = crate::providers::config::load().codex?;
    if entry.access_token.is_empty() || token_expired(&entry.access_token, 60) {
        return None;
    }
    Some(CodexTokens {
        access_token: entry.access_token,
        refresh_token: entry.refresh_token,
        source: entry.source,
    })
}

pub fn device_code_login() -> Result<CodexTokens, String> {
    let device = request_device_code()?;
    print_login_instructions(&device.user_code);
    let auth = poll_device_auth(&device.device_auth_id, &device.user_code, device.interval)?;
    exchange_tokens(&auth)
}

struct DeviceCode {
    user_code: String,
    device_auth_id: String,
    interval: u64,
}

struct DeviceAuth {
    authorization_code: String,
    code_verifier: String,
}

fn request_device_code() -> Result<DeviceCode, String> {
    let response = http_agent()
        .post(format!("{ISSUER}/api/accounts/deviceauth/usercode"))
        .header("Content-Type", "application/json")
        .send_json(serde_json::json!({ "client_id": CLIENT_ID }))
        .map_err(|error| error.to_string())?;
    let status = response.status().as_u16();
    let body = response.into_body().read_to_string().map_err(|error| error.to_string())?;
    if status != 200 {
        return Err(format!("device code request failed ({status}): {body}"));
    }
    let value: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    let user_code = field(&value, "user_code")?;
    let device_auth_id = field(&value, "device_auth_id")?;
    let interval = value["interval"].as_u64().unwrap_or(5).max(3);
    Ok(DeviceCode {
        user_code,
        device_auth_id,
        interval,
    })
}

fn poll_device_auth(device_auth_id: &str, user_code: &str, interval: u64) -> Result<DeviceAuth, String> {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(POLL_TIMEOUT_SECS) {
        thread::sleep(Duration::from_secs(interval));
        if let Some(auth) = poll_once(device_auth_id, user_code)? {
            return Ok(auth);
        }
    }
    Err("Codex login timed out after 15 minutes".to_owned())
}

fn poll_once(device_auth_id: &str, user_code: &str) -> Result<Option<DeviceAuth>, String> {
    let response = http_agent()
        .post(format!("{ISSUER}/api/accounts/deviceauth/token"))
        .header("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "device_auth_id": device_auth_id,
            "user_code": user_code,
        }))
        .map_err(|error| error.to_string())?;
    let status = response.status().as_u16();
    if status == 200 {
        let body = response.into_body().read_to_string().map_err(|error| error.to_string())?;
        let value: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
        return Ok(Some(DeviceAuth {
            authorization_code: field(&value, "authorization_code")?,
            code_verifier: field(&value, "code_verifier")?,
        }));
    }
    if status == 403 || status == 404 {
        return Ok(None);
    }
    Err(format!("device auth polling failed ({status})"))
}

fn exchange_tokens(auth: &DeviceAuth) -> Result<CodexTokens, String> {
    let response = http_agent()
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send_form([
            ("grant_type", "authorization_code"),
            ("code", auth.authorization_code.as_str()),
            ("redirect_uri", &format!("{ISSUER}/deviceauth/callback")),
            ("client_id", CLIENT_ID),
            ("code_verifier", auth.code_verifier.as_str()),
        ])
        .map_err(|error| error.to_string())?;
    let status = response.status().as_u16();
    let body = response.into_body().read_to_string().map_err(|error| error.to_string())?;
    if status != 200 {
        return Err(format!("token exchange failed ({status}): {body}"));
    }
    let value: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    let access_token = field(&value, "access_token")?;
    let refresh_token = value["refresh_token"].as_str().map(str::to_owned);
    Ok(CodexTokens {
        access_token,
        refresh_token,
        source: "device_code".to_owned(),
    })
}

fn print_login_instructions(user_code: &str) {
    println!("\nTo connect Codex (OpenAgents-style device login):\n");
    println!("  1. Open: {ISSUER}/codex/device");
    println!("  2. Enter code: {user_code}\n");
    println!("Waiting for sign-in...");
}

fn field(value: &Value, key: &str) -> Result<String, String> {
    value[key]
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("response missing '{key}'"))
}

fn codex_home() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("CODEX_HOME") {
        if !home.trim().is_empty() {
            return Some(PathBuf::from(home));
        }
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex"))
}

fn token_expired(token: &str, skew_secs: i64) -> bool {
    jwt_expiry(token)
        .is_some_and(|exp| exp <= unix_now() + skew_secs)
}

fn jwt_expiry(token: &str) -> Option<i64> {
    let payload = token.split('.').nth(1)?;
    let decoded = base64_decode(payload)?;
    let value: Value = serde_json::from_str(&decoded).ok()?;
    value["exp"].as_i64()
}

fn unix_now() -> i64 {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs()),
    )
    .unwrap_or(i64::MAX)
}

fn base64_decode(input: &str) -> Option<String> {
    let padded = match input.len() % 4 {
        2 => format!("{input}=="),
        3 => format!("{input}="),
        _ => input.to_owned(),
    };
    let bytes = decode_base64(&padded)?;
    String::from_utf8(bytes).ok()
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::new();
    let mut buffer = 0_u32;
    let mut bits = 0_u32;
    for ch in input.bytes() {
        if ch == b'=' {
            break;
        }
        let value = u32::try_from(TABLE.iter().position(|item| *item == ch)?).unwrap_or(0);
        buffer = (buffer << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(u8::try_from(buffer >> bits).unwrap_or(0));
            buffer &= (1 << bits) - 1;
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::{jwt_expiry, token_expired};

    #[test]
    fn detects_expired_jwt() {
        let token = "header.eyJleHAiOjF9.signature";
        assert!(token_expired(token, 0));
        assert_eq!(jwt_expiry(token), Some(1));
    }
}

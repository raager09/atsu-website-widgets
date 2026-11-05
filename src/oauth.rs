use crate::base64;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fmt::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

pub const AUTHORIZE_URL: &str = "https://atsu.moe/oauth/authorize";
pub const TOKEN_URL: &str = "https://atsu.moe/api/oauth/token";
pub const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8765/callback";
pub const SCOPE: &str = "history:read";

pub fn random_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| format!("the system random number generator failed: {e}"))?;
    Ok(base64::encode_url(&bytes))
}

pub fn challenge(verifier: &str) -> String {
    base64::encode_url(&Sha256::digest(verifier.as_bytes()))
}

pub fn authorize_url(client_id: &str, redirect_uri: &str, state: &str, challenge: &str) -> String {
    let query = form_encode(&[
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", SCOPE),
        ("state", state),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
    ]);
    format!("{AUTHORIZE_URL}?{query}")
}

pub fn form_encode(pairs: &[(&str, &str)]) -> String {
    let encoded: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect();
    encoded.join("&")
}

pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' => {
                let hex = bytes.get(i + 1..i + 3)?;
                if !hex.iter().all(u8::is_ascii_hexdigit) {
                    return None;
                }
                out.push(u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?);
                i += 2;
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8(out).ok()
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectError {
    Malformed,
    MissingCode,
    StateMismatch,
    Denied(String),
}

impl fmt::Display for RedirectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Malformed => f.write_str("the sign-in reply couldn't be read; copy the whole address and try again"),
            Self::MissingCode => f.write_str("the sign-in reply has no code; copy the whole address you were sent to, including everything after `?`"),
            Self::StateMismatch => f.write_str("the sign-in reply doesn't belong to this login attempt, so it was rejected; run login again"),
            Self::Denied(reason) => write!(f, "atsu.moe didn't grant access: {reason}"),
        }
    }
}

impl std::error::Error for RedirectError {}

pub fn code_from_redirect(reply: &str, expected_state: &str) -> Result<String, RedirectError> {
    let reply = reply.trim();
    let query = match reply.split_once('?') {
        Some((_, query)) => query,
        None if reply.contains("://") || reply.starts_with('/') => "",
        None => reply,
    };
    let query = query.split('#').next().unwrap_or_default();

    let [mut code, mut state, mut error, mut description] = [None, None, None, None];
    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let slot = match percent_decode(key).as_deref() {
            Some("code") => &mut code,
            Some("state") => &mut state,
            Some("error") => &mut error,
            Some("error_description") => &mut description,
            Some(_) => continue,
            None => return Err(RedirectError::Malformed),
        };
        if slot.is_some() {
            return Err(RedirectError::Malformed);
        }
        *slot = Some(percent_decode(value).ok_or(RedirectError::Malformed)?);
    }

    if code.is_none() && state.is_none() && error.is_none() {
        return Err(RedirectError::MissingCode);
    }
    if state.as_deref() != Some(expected_state) {
        return Err(RedirectError::StateMismatch);
    }
    if let Some(error) = error {
        let reason = match description {
            Some(description) => format!("{error} ({description})"),
            None => error,
        };
        return Err(RedirectError::Denied(printable(&reason)));
    }
    code.filter(|c| !c.is_empty())
        .ok_or(RedirectError::MissingCode)
}

fn printable(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(200).collect()
}

#[derive(Debug, PartialEq, Eq)]
pub enum Route {
    Loopback { addr: SocketAddr, path: String },
    Paste,
}

pub fn route(redirect_uri: &str) -> Route {
    let rest = match redirect_uri.get(..7) {
        Some(scheme) if scheme.eq_ignore_ascii_case("http://") => &redirect_uri[7..],
        _ => return Route::Paste,
    };
    let (authority, path) = match rest.find(['/', '?', '#']) {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let path = if path.is_empty() { "/" } else { path };

    let (host, port) = match authority.strip_prefix('[') {
        Some(v6) => match v6.split_once(']') {
            Some((host, "")) => (host, None),
            Some((host, port)) => match port.strip_prefix(':') {
                Some(port) => (host, Some(port)),
                None => return Route::Paste,
            },
            None => return Route::Paste,
        },
        None => match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        },
    };
    let port = match port.map(str::parse::<u16>) {
        None => 80,
        Some(Ok(port)) if port != 0 => port,
        _ => return Route::Paste,
    };
    let ip = if host.eq_ignore_ascii_case("localhost") {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        match host.parse::<IpAddr>() {
            Ok(ip) if ip.is_loopback() => ip,
            _ => return Route::Paste,
        }
    };
    Route::Loopback {
        addr: SocketAddr::new(ip, port),
        path: path.to_string(),
    }
}

pub fn token_form<'a>(
    client_id: &'a str,
    client_secret: &'a str,
    code: &'a str,
    redirect_uri: &'a str,
    verifier: &'a str,
) -> [(&'static str, &'a str); 6] {
    [
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier),
    ]
}

#[derive(Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub scope: Option<String>,
}

impl fmt::Debug for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Tokens")
            .field("access_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .field("scope", &self.scope)
            .finish()
    }
}

pub fn parse_tokens(body: &str) -> Result<Tokens, String> {
    match serde_json::from_str::<Tokens>(body) {
        Ok(tokens) if !tokens.access_token.is_empty() && !tokens.refresh_token.is_empty() => {
            Ok(tokens)
        }
        _ => Err("atsu.moe sent a token response this tool can't read".to_string()),
    }
}

pub fn token_error(status: u16) -> String {
    match status {
        400 => "atsu.moe rejected the token request (HTTP 400). The code or refresh token has expired, was already used, or belongs to a different app or redirect URI.".to_string(),
        401 => "atsu.moe rejected the client id or secret (HTTP 401). Check ATSU_CLIENT_ID and ATSU_CLIENT_SECRET.".to_string(),
        403 => "atsu.moe refused the request (HTTP 403). The app may be missing the history:read scope or still be in review.".to_string(),
        429 => "atsu.moe is rate limiting requests (HTTP 429). Wait a minute and try again.".to_string(),
        _ => format!("the atsu.moe token request failed (HTTP {status})"),
    }
}

pub fn request_tokens(form: &[(&str, &str)]) -> Result<Tokens, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .user_agent(concat!("atsu-shelf/", env!("CARGO_PKG_VERSION")))
        .build();
    match agent.post(TOKEN_URL).send_form(form) {
        Ok(response) => {
            let body = response
                .into_string()
                .map_err(|_| "the atsu.moe token response couldn't be read".to_string())?;
            parse_tokens(&body)
        }
        Err(ureq::Error::Status(status, _)) => Err(token_error(status)),
        Err(ureq::Error::Transport(transport)) => {
            Err(format!("couldn't reach atsu.moe: {}", transport.kind()))
        }
    }
}

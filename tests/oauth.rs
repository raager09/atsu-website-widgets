use atsu_shelf::base64;
use atsu_shelf::login;
use atsu_shelf::oauth::{self, RedirectError, Route};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};

#[test]
fn challenge_matches_rfc_7636() {
    assert_eq!(
        oauth::challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[test]
fn base64url_is_url_safe_and_unpadded() {
    assert_eq!(base64::encode_url(&[0xfb, 0xff]), "-_8");
    assert_eq!(base64::encode_url(b"a"), "YQ");
    assert_eq!(base64::encode_url(b"ab"), "YWI");
    assert_eq!(base64::encode_url(b"abc"), "YWJj");
    assert_eq!(base64::encode(&[0xfb, 0xff]), "+/8=");
}

#[test]
fn random_tokens_are_valid_verifiers_and_differ() {
    let a = oauth::random_token().unwrap();
    let b = oauth::random_token().unwrap();
    assert_ne!(a, b);
    for token in [a, b] {
        assert_eq!(token.len(), 43);
        assert!(
            token
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        );
    }
}

#[test]
fn authorize_url_is_encoded() {
    assert_eq!(
        oauth::authorize_url(
            "my app&x=1",
            "http://127.0.0.1:8765/callback",
            "st-_.~",
            "ch"
        ),
        "https://atsu.moe/oauth/authorize?client_id=my%20app%26x%3D1\
         &redirect_uri=http%3A%2F%2F127.0.0.1%3A8765%2Fcallback\
         &response_type=code&scope=history%3Aread&state=st-_.~\
         &code_challenge=ch&code_challenge_method=S256"
    );
}

#[test]
fn token_form_has_every_field() {
    let form = oauth::token_form(
        "id",
        "secret",
        "code",
        "http://127.0.0.1:8765/callback",
        "verifier",
    );
    assert_eq!(
        form,
        [
            ("grant_type", "authorization_code"),
            ("client_id", "id"),
            ("client_secret", "secret"),
            ("code", "code"),
            ("redirect_uri", "http://127.0.0.1:8765/callback"),
            ("code_verifier", "verifier"),
        ]
    );
}

#[test]
fn redirect_code_is_percent_decoded() {
    let reply = "http://127.0.0.1:8765/callback?code=a%2Fb%2Bc+d&state=s%3D1#frag";
    assert_eq!(oauth::code_from_redirect(reply, "s=1").unwrap(), "a/b+c d");
}

#[test]
fn redirect_accepts_a_bare_query_and_surrounding_whitespace() {
    assert_eq!(
        oauth::code_from_redirect("  code=xyz&state=s\r\n", "s").unwrap(),
        "xyz"
    );
    assert_eq!(
        oauth::code_from_redirect("/callback?state=s&code=xyz", "s").unwrap(),
        "xyz"
    );
}

#[test]
fn redirect_without_a_code_is_rejected() {
    for reply in [
        "http://127.0.0.1:8765/callback",
        "http://127.0.0.1:8765/callback?",
        "http://127.0.0.1:8765/callback?state=s",
        "http://127.0.0.1:8765/callback?state=s&code=",
        "",
    ] {
        assert_eq!(
            oauth::code_from_redirect(reply, "s"),
            Err(RedirectError::MissingCode),
            "{reply}"
        );
    }
}

#[test]
fn redirect_with_the_wrong_state_is_rejected() {
    for reply in ["?code=c&state=other", "?code=c", "?code=c&state="] {
        assert_eq!(
            oauth::code_from_redirect(reply, "s"),
            Err(RedirectError::StateMismatch),
            "{reply}"
        );
    }
}

#[test]
fn redirect_error_is_reported_without_control_characters() {
    let reply = "?error=access_denied&error_description=User+said+no%1B%5B31m&state=s";
    assert_eq!(
        oauth::code_from_redirect(reply, "s"),
        Err(RedirectError::Denied(
            "access_denied (User said no[31m)".to_string()
        ))
    );
    assert_eq!(
        oauth::code_from_redirect("?error=access_denied&state=s", "s"),
        Err(RedirectError::Denied("access_denied".to_string()))
    );
}

#[test]
fn forged_error_with_the_wrong_state_reports_a_mismatch() {
    assert_eq!(
        oauth::code_from_redirect("?error=please+visit+evil.example&state=x", "s"),
        Err(RedirectError::StateMismatch)
    );
}

#[test]
fn malformed_redirects_are_rejected() {
    for reply in [
        "?code=%zz&state=s",
        "?code=%4&state=s",
        "?code=%+1&state=s",
        "?code=a&code=b&state=s",
        "?code=%FF&state=s",
    ] {
        assert_eq!(
            oauth::code_from_redirect(reply, "s"),
            Err(RedirectError::Malformed),
            "{reply}"
        );
    }
}

fn loopback(ip: impl Into<std::net::IpAddr>, port: u16, path: &str) -> Route {
    Route::Loopback {
        addr: SocketAddr::new(ip.into(), port),
        path: path.to_string(),
    }
}

#[test]
fn loopback_redirects_are_caught_locally() {
    let v4 = Ipv4Addr::LOCALHOST;
    for (uri, want) in [
        (oauth::DEFAULT_REDIRECT_URI, loopback(v4, 8765, "/callback")),
        ("http://localhost:9000/cb", loopback(v4, 9000, "/cb")),
        ("HTTP://LocalHost:9000/cb?x=1", loopback(v4, 9000, "/cb")),
        ("http://127.0.0.1", loopback(v4, 80, "/")),
        ("http://127.0.0.1:8080?x", loopback(v4, 8080, "/")),
        (
            "http://[::1]:7000/a/b",
            loopback(Ipv6Addr::LOCALHOST, 7000, "/a/b"),
        ),
    ] {
        assert_eq!(oauth::route(uri), want, "{uri}");
    }
}

#[test]
fn other_redirects_ask_for_a_paste() {
    for uri in [
        "https://127.0.0.1:8765/callback",
        "https://example.com/callback",
        "http://example.com/callback",
        "http://192.168.1.2:8765/callback",
        "http://127.0.0.1:0/callback",
        "http://127.0.0.1:99999/callback",
        "http://127.0.0.1:/callback",
        "http://user@127.0.0.1/callback",
        "http://[::1/callback",
        "http://[::1]x/callback",
        "myapp://callback",
        "",
    ] {
        assert_eq!(oauth::route(uri), Route::Paste, "{uri}");
    }
}

fn get(addr: SocketAddr, request: &str) -> String {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn listener_ignores_other_requests_and_returns_the_callback() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let waiting = std::thread::spawn(move || login::catch(&listener, "/callback"));

    assert!(get(addr, "GET /favicon.ico HTTP/1.1\r\nHost: x\r\n\r\n").starts_with("HTTP/1.1 404"));
    assert!(get(addr, "GET /callbackx?code=c HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 404"));
    assert!(get(addr, "POST /callback?code=c HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 400"));
    assert!(get(addr, "garbage\r\n\r\n").starts_with("HTTP/1.1 400"));
    let ok = get(
        addr,
        "GET /callback?code=c&state=s HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
    );
    assert!(ok.starts_with("HTTP/1.1 200"), "{ok}");
    assert!(ok.contains("close this tab"));

    assert_eq!(waiting.join().unwrap().unwrap(), "/callback?code=c&state=s");
}

#[test]
fn token_response_is_parsed_and_debug_hides_it() {
    let tokens = oauth::parse_tokens(
        r#"{"access_token":"access-123","refresh_token":"refresh-456","expires_in":3600,"scope":"history:read","token_type":"Bearer"}"#,
    )
    .unwrap();
    assert_eq!(tokens.access_token, "access-123");
    assert_eq!(tokens.refresh_token, "refresh-456");
    assert_eq!(tokens.scope.as_deref(), Some("history:read"));
    let debug = format!("{tokens:?}");
    assert!(
        !debug.contains("access-123") && !debug.contains("refresh-456"),
        "{debug}"
    );
}

#[test]
fn bad_token_responses_do_not_echo_the_body() {
    for body in [
        "",
        "not json",
        r#"{"error":"invalid_grant"}"#,
        r#"{"access_token":"secret-a","refresh_token":""}"#,
        r#"{"access_token":"secret-a","refresh_token":12345}"#,
    ] {
        let error = oauth::parse_tokens(body).unwrap_err();
        assert!(
            !error.contains("secret-a")
                && !error.contains("12345")
                && !error.contains("invalid_grant"),
            "{error}"
        );
    }
}

#[test]
fn token_errors_explain_the_status() {
    assert!(oauth::token_error(400).contains("expired"));
    assert!(oauth::token_error(401).contains("ATSU_CLIENT_SECRET"));
    assert!(oauth::token_error(403).contains("history:read"));
    assert!(oauth::token_error(429).contains("Wait"));
    assert!(oauth::token_error(502).contains("502"));
}

use crate::oauth::{self, Route};
use crate::token_file;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

pub struct Login {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub save: Option<PathBuf>,
}

pub fn run(login: &Login) -> Result<(), String> {
    let verifier = oauth::random_token()?;
    let state = oauth::random_token()?;
    let url = oauth::authorize_url(
        &login.client_id,
        &login.redirect_uri,
        &state,
        &oauth::challenge(&verifier),
    );

    let reply = match oauth::route(&login.redirect_uri) {
        Route::Loopback { addr, path } => {
            let listener = TcpListener::bind(addr).map_err(|e| {
                format!(
                    "can't listen on {addr} for the sign-in reply ({e}); is another login running?"
                )
            })?;
            eprintln!("Open this link to sign in to atsu.moe:\n\n{url}\n");
            eprintln!(
                "Waiting for atsu.moe to send you back to {} ...",
                login.redirect_uri
            );
            catch(&listener, &path).map_err(|e| format!("the sign-in listener failed: {e}"))?
        }
        Route::Paste => {
            eprintln!("Open this link to sign in to atsu.moe:\n\n{url}\n");
            eprintln!(
                "After you approve, your browser goes to {}. That page may not load, which is fine.",
                login.redirect_uri
            );
            eprint!("Copy the full address from the address bar and paste it here: ");
            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .map_err(|e| format!("couldn't read the pasted address: {e}"))?;
            line
        }
    };

    let code = oauth::code_from_redirect(&reply, &state).map_err(|e| e.to_string())?;
    let form = oauth::token_form(
        &login.client_id,
        &login.client_secret,
        &code,
        &login.redirect_uri,
        &verifier,
    );
    let tokens = oauth::request_tokens(&form)?;

    match &login.save {
        Some(path) => {
            token_file::write(path, &tokens.refresh_token)
                .map_err(|e| format!("can't write {}: {e}", path.display()))?;
            eprintln!("Signed in. The refresh token is in {}.", path.display());
            if cfg!(not(unix)) {
                eprintln!(
                    "note: on Windows the file keeps your folder's default permissions; delete it once you've copied the token."
                );
            }
        }
        None => {
            eprintln!("Signed in. Your refresh token:");
            println!("{}", tokens.refresh_token);
        }
    }
    eprintln!(
        "Store it as the ATSU_REFRESH_TOKEN secret. It works once: each run swaps it for a new one."
    );
    Ok(())
}

pub fn catch(listener: &TcpListener, path: &str) -> io::Result<String> {
    loop {
        let (stream, _) = listener.accept()?;
        if let Some(target) = answer(stream, path) {
            return Ok(target);
        }
    }
}

fn answer(mut stream: TcpStream, path: &str) -> Option<String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok()?;
    let mut reader = BufReader::new((&stream).take(16 * 1024));
    let mut request_line = String::new();
    reader.read_line(&mut request_line).ok()?;
    let mut header = String::new();
    while reader.read_line(&mut header).is_ok_and(|n| n > 0) && header.trim_end() != "" {
        header.clear();
    }

    let target = match request_line.split(' ').collect::<Vec<_>>()[..] {
        ["GET", target, version] if version.trim_end().starts_with("HTTP/") => target,
        _ => {
            respond(&mut stream, "400 Bad Request", "Bad request.");
            return None;
        }
    };
    if target.split(['?', '#']).next() != Some(path) {
        respond(&mut stream, "404 Not Found", "Not found.");
        return None;
    }
    respond(
        &mut stream,
        "200 OK",
        "atsu-shelf got the reply from atsu.moe. You can close this tab; the terminal shows whether sign-in worked.",
    );
    Some(target.to_string())
}

fn respond(stream: &mut TcpStream, status: &str, message: &str) {
    let body = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>atsu-shelf</title><p style=\"font:16px system-ui,sans-serif;margin:3em\">{message}</p>"
    );
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.flush();
}

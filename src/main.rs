use atsu_shelf::card::{self, Color, Options};
use atsu_shelf::login::{self, Login};
use atsu_shelf::{demo, oauth};
use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "atsu-shelf", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render the widget to an SVG file
    Draw(DrawArgs),
    /// Sign in to atsu.moe once and get a refresh token; reads ATSU_CLIENT_SECRET
    Login(LoginArgs),
}

#[derive(Args)]
struct LoginArgs {
    /// Your atsu.moe app's client id
    #[arg(long, env = "ATSU_CLIENT_ID", hide_env_values = true)]
    client_id: String,
    /// Must exactly match a redirect URI registered on your app
    #[arg(long, default_value = oauth::DEFAULT_REDIRECT_URI)]
    redirect_uri: String,
    /// Write the refresh token to this file instead of printing it
    #[arg(long)]
    save: Option<PathBuf>,
}

#[derive(Args)]
struct DrawArgs {
    /// Use built-in invented titles, with no network access
    #[arg(long)]
    demo: bool,
    #[arg(long, default_value = "shelf.svg")]
    out: PathBuf,
    #[arg(long, default_value = "Currently reading")]
    heading: String,
    /// How many books to show
    #[arg(long, default_value_t = 6, value_parser = clap::value_parser!(u8).range(1..=12))]
    cards: u8,
    #[arg(long)]
    heading_color: Option<Color>,
    #[arg(long)]
    title_color: Option<Color>,
    #[arg(long)]
    background: Option<Color>,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Draw(args) => draw(args),
        Command::Login(args) => sign_in(args),
    }
}

fn sign_in(args: LoginArgs) -> ExitCode {
    let client_secret = match std::env::var("ATSU_CLIENT_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => secret.trim().to_string(),
        _ => {
            eprintln!(
                "error: set the ATSU_CLIENT_SECRET environment variable to your app's client secret (it isn't a flag, so it stays out of your shell history)"
            );
            return ExitCode::from(2);
        }
    };
    let result = login::run(&Login {
        client_id: args.client_id.trim().to_string(),
        client_secret,
        redirect_uri: args.redirect_uri,
        save: args.save,
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn draw(args: DrawArgs) -> ExitCode {
    if !args.demo {
        eprintln!("error: reading from atsu.moe isn't built yet; run with --demo");
        return ExitCode::from(2);
    }

    let opts = Options {
        heading: args.heading,
        cards: args.cards as usize,
        heading_color: args.heading_color,
        title_color: args.title_color,
        background: args.background,
    };
    let svg = card::render(&demo::books(), &opts);
    match write(&args.out, &svg) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: can't write {}: {e}", args.out.display());
            ExitCode::FAILURE
        }
    }
}

fn write(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)
}

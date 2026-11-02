use atsu_shelf::card::{self, Color, Options};
use atsu_shelf::demo;
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
    let Command::Draw(args) = Cli::parse().command;
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

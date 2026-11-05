use std::path::PathBuf;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_atsu-shelf"))
        .args(args)
        .output()
        .unwrap()
}

fn tmp(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn demo_writes_an_svg_and_creates_parent_folders() {
    let out = tmp("nested/dir/demo.svg");
    let _ = std::fs::remove_dir_all(tmp("nested"));
    let result = run(&["draw", "--demo", "--out", out.to_str().unwrap()]);
    assert!(result.status.success(), "{result:?}");
    let svg = std::fs::read_to_string(&out).unwrap();
    assert!(svg.starts_with("<svg "));
    assert_eq!(svg.matches("class=\"card\"").count(), 6);
}

#[test]
fn flags_reach_the_output() {
    let out = tmp("themed.svg");
    let result = run(&[
        "draw",
        "--demo",
        "--out",
        out.to_str().unwrap(),
        "--heading",
        "Currently reading",
        "--cards",
        "2",
        "--heading-color",
        "#222222",
        "--title-color",
        "#ff69b4",
        "--background",
        "#ffffff",
    ]);
    assert!(result.status.success(), "{result:?}");
    let svg = std::fs::read_to_string(&out).unwrap();
    assert!(svg.contains(">Currently reading<"));
    assert_eq!(svg.matches("class=\"card\"").count(), 2);
    assert!(svg.contains("var(--atsu-shelf-title,#ff69b4)"));
}

#[test]
fn bad_flags_are_rejected() {
    for args in [
        &["draw", "--demo", "--cards", "0"][..],
        &["draw", "--demo", "--cards", "13"][..],
        &["draw", "--demo", "--title-color", "pink;}"][..],
    ] {
        assert!(!run(args).status.success(), "{args:?}");
    }
}

fn login(args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atsu-shelf"));
    command
        .arg("login")
        .args(args)
        .env_remove("ATSU_CLIENT_ID")
        .env_remove("ATSU_CLIENT_SECRET");
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().unwrap()
}

#[test]
fn login_needs_a_client_id() {
    let result = login(&[], &[("ATSU_CLIENT_SECRET", "s3cret-value")]);
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("--client-id"), "{stderr}");
}

#[test]
fn login_reads_the_secret_only_from_the_environment() {
    for env in [&[][..], &[("ATSU_CLIENT_SECRET", "  ")][..]] {
        let result = login(&["--client-id", "abc"], env);
        assert_eq!(result.status.code(), Some(2));
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains("ATSU_CLIENT_SECRET"), "{stderr}");
        assert!(!stderr.contains("atsu.moe/oauth"), "{stderr}");
    }
    let flag = login(
        &["--client-id", "abc", "--client-secret", "s3cret-value"],
        &[],
    );
    assert!(!flag.status.success());
    assert!(!String::from_utf8_lossy(&flag.stderr).contains("s3cret-value"));
}

#[test]
fn login_help_hides_the_client_id_value() {
    let result = login(&["--help"], &[("ATSU_CLIENT_ID", "my-private-id")]);
    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("ATSU_CLIENT_ID") && !stdout.contains("my-private-id"),
        "{stdout}"
    );
}

#[test]
fn live_mode_fails_clearly_until_it_is_built() {
    let result = run(&["draw"]);
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("--demo"), "{stderr}");
}

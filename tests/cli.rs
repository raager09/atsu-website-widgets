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

#[test]
fn live_mode_fails_clearly_until_it_is_built() {
    let result = run(&["draw"]);
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("--demo"), "{stderr}");
}

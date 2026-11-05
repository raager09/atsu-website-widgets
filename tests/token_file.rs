use atsu_shelf::token_file;
use std::path::PathBuf;

fn dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn entries(dir: &PathBuf) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn writes_and_replaces_the_token_without_leftovers() {
    let dir = dir("token-replace");
    let path = dir.join("refresh-token");
    token_file::write(&path, "first").unwrap();
    token_file::write(&path, "second").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
    assert_eq!(entries(&dir), ["refresh-token"]);
}

#[test]
fn a_stale_temp_file_is_replaced() {
    let dir = dir("token-stale");
    std::fs::write(dir.join(".refresh-token.tmp"), "half-writ").unwrap();
    token_file::write(&dir.join("refresh-token"), "fresh").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("refresh-token")).unwrap(),
        "fresh"
    );
    assert_eq!(entries(&dir), ["refresh-token"]);
}

#[test]
fn a_failed_write_keeps_the_old_token() {
    let dir = dir("token-fail");
    let path = dir.join("refresh-token");
    token_file::write(&path, "old").unwrap();
    assert!(token_file::write(&dir.join("missing").join("refresh-token"), "new").is_err());
    assert!(token_file::write(&dir, "new").is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "old");
}

#[cfg(unix)]
#[test]
fn the_file_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = dir("token-mode");
    let path = dir.join("refresh-token");
    token_file::write(&path, "secret").unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

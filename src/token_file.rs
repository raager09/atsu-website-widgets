use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn write(path: &Path, token: &str) -> io::Result<()> {
    let tmp = temp_path(path)?;
    let _ = fs::remove_file(&tmp);
    let written = create_private(&tmp).and_then(|mut file| {
        file.write_all(token.as_bytes())?;
        file.sync_all()
    });
    if let Err(e) = written.and_then(|()| fs::rename(&tmp, path)) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

fn temp_path(path: &Path) -> io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "not a file path"))?;
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(".tmp");
    Ok(path.with_file_name(tmp_name))
}

#[cfg(unix)]
fn create_private(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn create_private(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

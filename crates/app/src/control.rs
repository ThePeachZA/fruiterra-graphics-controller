//! Local control token persistence.
//!
//! The token lives beside `app.json` in a plain file so the v1 state schema is
//! unchanged. Writes go through a temporary file and rename, and the file is
//! never included in project bundles or backups.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const TOKEN_FILE: &str = "control-token";

/// Reads the saved token or generates and saves one when it is missing.
pub(crate) fn load_or_create_token() -> Result<String, String> {
    fgc_core::ensure_app_dirs().map_err(|error| error.to_string())?;
    let path = fgc_core::data_dir()
        .map_err(|error| error.to_string())?
        .join(TOKEN_FILE);
    if let Some(token) = read_token_at(&path)? {
        return Ok(token);
    }

    let token = fgc_server::generate_control_token();
    write_token_at(&path, &token)?;
    Ok(token)
}

/// Generates a replacement token and saves it atomically.
pub(crate) fn create_token() -> Result<String, String> {
    fgc_core::ensure_app_dirs().map_err(|error| error.to_string())?;
    let path = fgc_core::data_dir()
        .map_err(|error| error.to_string())?
        .join(TOKEN_FILE);
    let token = fgc_server::generate_control_token();
    write_token_at(&path, &token)?;
    Ok(token)
}

fn read_token_at(path: &Path) -> Result<Option<String>, String> {
    let bytes = match fs::read_to_string(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("could not read control token: {error}")),
    };
    let token = bytes.trim();
    if is_token(token) {
        Ok(Some(token.to_owned()))
    } else {
        Ok(None)
    }
}

fn write_token_at(path: &Path, token: &str) -> Result<(), String> {
    if !is_token(token) {
        return Err("generated control token has an invalid shape".to_owned());
    }

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "control token path has no file name".to_owned())?;
    let temporary = path.with_file_name(format!("{name}.tmp-{}", std::process::id()));
    let _ = fs::remove_file(&temporary);

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("could not create control token file: {error}"))?;
    file.write_all(token.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("could not write control token: {error}"))?;
    drop(file);

    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("could not replace control token: {error}"))?;
    }
    fs::rename(&temporary, path)
        .map_err(|error| format!("could not install control token: {error}"))
}

fn is_token(token: &str) -> bool {
    let bytes = token.as_bytes();
    if bytes.len() != 36
        || bytes[8] != b'-'
        || bytes[13] != b'-'
        || bytes[18] != b'-'
        || bytes[23] != b'-'
        || bytes[14] != b'4'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
    {
        return false;
    }
    bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_have_a_valid_shape() {
        let token = fgc_server::generate_control_token();
        assert!(is_token(&token));
    }

    #[test]
    fn tokens_without_a_v4_version_are_rejected() {
        let mut bytes = fgc_server::generate_control_token().into_bytes();
        bytes[14] = b'1';
        let token = String::from_utf8(bytes).expect("valid UTF-8 token");
        assert!(!is_token(&token));
    }

    #[test]
    fn token_round_trips_through_disk() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("control-token");
        let token = fgc_server::generate_control_token();

        write_token_at(&path, &token).expect("write token");
        let loaded = read_token_at(&path).expect("read token");

        assert_eq!(loaded.as_deref(), Some(token.as_str()));
    }

    #[test]
    fn missing_token_file_loads_as_none() {
        let directory = tempfile::tempdir().expect("temp directory");
        let loaded = read_token_at(&directory.path().join("control-token")).expect("read token");
        assert_eq!(loaded, None);
    }
}

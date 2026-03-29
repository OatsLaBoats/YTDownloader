use std::ffi::OsStr;
use std::sync::Arc;

use thiserror::Error;
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::platform::windows::convert_ascii_to_utf8;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("failed to spawn deno process")]
    SpawnDenoFailed(Arc<std::io::Error>),

    #[error("deno returned a non zero exit code")]
    DenoCommandFailed,

    #[error("failed to convert bytes to utf8")]
    ConvertBytesToUTF8Failed,
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn query_version(deno_path: impl AsRef<OsStr>) -> Result<String> {
    let result = tokio::process::Command::new(deno_path)
        .creation_flags(CREATE_NO_WINDOW.0)
        .kill_on_drop(true)
        .arg("--version")
        .output().await.map_err(|e|
            Error::SpawnDenoFailed(Arc::new(e))
        )?;

    let output = result.stdout;
    let status = result.status;

    if !status.success() {
        return Err(Error::DenoCommandFailed);
    }

    let mut end_index = 10;

    // Handle the minor version being in the double digits
    if (output[end_index - 2] as char).is_ascii_digit() {
        end_index += 1;
    }

    // Handle the patch version being in the double digits
    if (output[end_index] as char).is_ascii_digit() {
        end_index += 1;
    }

    // Cut the /r/n at the end
    let version_slice = convert_ascii_to_utf8(&output[5..end_index]).ok_or(()).map_err(|_|
        Error::ConvertBytesToUTF8Failed
    )?;

    let version = version_slice.trim_end_matches(&['\r', '\n']);

    let mut version_string = version.to_string();
    version_string.insert(0, 'v');

    Ok(version_string)
}

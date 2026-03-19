use std::ffi::OsStr;
use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("failed to spawn deno process")]
    SpawnDenoFailed(Arc<std::io::Error>),

    #[error("failed to convert bytes to utf8")]
    ConvertBytesToUTF8Failed,
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn query_version(deno_path: impl AsRef<OsStr>) -> Result<String> {
    let result = tokio::process::Command::new(deno_path)
        .kill_on_drop(true)
        .arg("--version")
        .output().await.map_err(|e|
            Error::SpawnDenoFailed(Arc::new(e))
        )?
        .stdout;

    let version_slice = str::from_utf8(&result[5..10]).map_err(|_|
        Error::ConvertBytesToUTF8Failed
    )?;

    let mut version_string = version_slice.to_string();
    version_string.insert(0, 'v');

    Ok(version_string)
}

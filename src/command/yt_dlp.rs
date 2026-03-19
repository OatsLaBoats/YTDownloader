use std::ffi::OsStr;
use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("failed to spawn yt-dlp process")]
    SpawnYtDlpFailed(Arc<std::io::Error>),

    #[error("failed to convert bytes to utf8")]
    ConvertBytesToUTF8Failed,
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn query_version(yt_dlp_path: impl AsRef<OsStr>) -> Result<String> {
    let result = tokio::process::Command::new(yt_dlp_path)
        .kill_on_drop(true)
        .arg("--version")
        .output().await.map_err(|e|
            Error::SpawnYtDlpFailed(Arc::new(e))
        )?
        .stdout;

    
    
    // Cut the /r/n at the end
    let version_slice = str::from_utf8(&result).map_err(|_|
        Error::ConvertBytesToUTF8Failed
    )?.trim_end_matches(&['\r', '\n']);

    Ok(version_slice.to_string())
}

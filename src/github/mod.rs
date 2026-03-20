use std::sync::Arc;

use thiserror::Error;
use reqwest::Client;
use reqwest::header::*;
use serde::{Serialize, Deserialize};

type Result<T> = std::result::Result<T, Error>;

pub const YT_DLP_OWNER: &str = "yt-dlp";
pub const YT_DLP_REPO: &str = "yt-dlp";

pub const FFMPEG_OWNER: &str = "yt-dlp";
pub const FFMPEG_REPO: &str = "FFmpeg-Builds";

pub const DENO_OWNER: &str = "denoland";
pub const DENO_REPO: &str = "deno";

pub const API_VERSION: &str = "2026-03-10";

pub async fn query_latest_release(client: Client, owner: &str, repo: &str) -> Result<LatestRelease> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let response = client
        .get(url)
        .header("X-GitHub-Api-Version", API_VERSION)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "rust-web-api-client")
        .send().await.map_err(|e|
            Error::QueryLatestReleaseFailed(Arc::new(e))
        )?;

    let json = response.json().await.map_err(|e|
        Error::ParseJsonFailed(Arc::new(e))
    )?;

    return Ok(json);
}

pub async fn start_asset_download(client: Client, asset_url: &str) -> Result<reqwest::Response> {
    client
        .get(asset_url)
        .header("X-GitHub-Api-Version", API_VERSION)
        .header(ACCEPT, "application/octet-stream")
        .header(USER_AGENT, "rust-web-api-client")
        .send().await.map_err(|e| Error::GetReleaseAssetFailed(Arc::new(e)))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LatestRelease {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    pub name: String,
    pub url: String,
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("failed to get latest release with the github api")]
    QueryLatestReleaseFailed(Arc<reqwest::Error>),

    #[error("failed to get release asset with the github api")]
    GetReleaseAssetFailed(Arc<reqwest::Error>),

    #[error("failed to parse json returned by the github api")]
    ParseJsonFailed(Arc<reqwest::Error>),
}

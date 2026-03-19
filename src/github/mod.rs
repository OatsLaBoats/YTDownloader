use std::sync::Arc;

use thiserror::Error;
use reqwest::Client;
use reqwest::header::*;
use serde::{Serialize, Deserialize};

type Result<T> = std::result::Result<T, GithubError>;

pub const YT_DLP_OWNER: &'static str = "yt-dlp";
pub const YT_DLP_REPO: &'static str = "yt-dlp";

pub async fn get_latest_release(client: Client, owner: &str, repo: &str) -> Result<LatestRelease> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let response = client
        .get(url)
        .header("X-GitHub-Api-Version", "2026-03-10")
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "rust-web-api-client")
        .send().await.map_err(|e|
            GithubError::QueryLatestReleaseFailed(Arc::new(e))
        )?;

    let json = response.json().await.map_err(|e|
        GithubError::ParseJsonFailed(Arc::new(e))
    )?;

    return Ok(json);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LatestRelease {
    pub tag_name: String,
}

#[derive(Error, Debug, Clone)]
pub enum GithubError {
    #[error("failed to get latest release the github api")]
    QueryLatestReleaseFailed(Arc<reqwest::Error>),

    #[error("failed to parse json returned by the github api")]
    ParseJsonFailed(Arc<reqwest::Error>),
}

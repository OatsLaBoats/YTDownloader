use reqwest::*;
use reqwest::header::*;
use serde::{Serialize, Deserialize};

pub async fn get_latest_yt_dlp_release(client: Client) -> anyhow::Result<LatestRelease> {
    let url = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";
    let response = client
        .get(url)
        .header("X-GitHub-Api-Version", "2026-03-10")
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "rust-web-api-client")
        .send()
        .await?;

    let json = response.json().await?;

    return Ok(json);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LatestRelease {
    pub tag_name: String,
}

use std::sync::Arc;

use reqwest::Client;
use tracing::error;
use tracing::info;

use crate::Paths;
use crate::github;
use crate::command::yt_dlp;
use crate::command::deno;


pub async fn open_file_picker(cwd: String) -> Option<String> {
    Some(rfd::AsyncFileDialog::new()
        .set_directory(&cwd)
        .pick_folder()
        .await?
        .path()
        .to_string_lossy()
        .to_string())
}

pub async fn check_for_updates(paths: Arc<Paths>, client: Client) -> bool {
    info!("CHECK_FOR_UPDATE: start");

    if !paths.bin_dir.exists() {
        info!("CHECK_FOR_UPDATE: bin directory missing");
        return true;
    }

    if !paths.yt_dlp_exe.exists() {
        info!("CHECK_FOR_UPDATE: yt-dlp executable missing");
        return true;
    }

    if !paths.ffmpeg_dir.exists() {
        info!("CHECK_FOR_UPDATE: ffmpeg directory missing");
        return true;
    }

    if !paths.deno_exe.exists() {
        info!("CHECK_FOR_UPDATE: deno executable missing");
        return true;
    }

    // Unlike the updater we just return false on failure since it's not important
    let latest_yt_dlp_release = match github::query_latest_release(
        client.clone(),
        github::YT_DLP_OWNER,
        github::YT_DLP_REPO,
    ).await {
        Ok(v) => v,
        Err(e) => {
            error!("CHECK_FOR_UPDATE: failed to query latest yt-dlp release {e}");
            return false;
        },
    };
    
    let current_yt_dlp_version = match yt_dlp::query_version(&paths.yt_dlp_exe).await {
        Ok(v) => v,
        Err(e) => {
            error!("CHECK_FOR_UPDATE: failed to query local yt-dlp version {e}");
            return false;
        },
    };

    if current_yt_dlp_version != latest_yt_dlp_release.tag_name {
        info!("CHECK_FOR_UPDATE: yt-dlp is outdated");
        return true;
    }

    let latest_deno_release = match github::query_latest_release(
        client.clone(),
        github::DENO_OWNER,
        github::DENO_REPO,
    ).await {
        Ok(v) => v,
        Err(e) => {
            error!("CHECK_FOR_UPDATE: failed to query latest deno release {e}");
            return false;
        },
    };
    
    let current_deno_version = match deno::query_version(&paths.deno_exe).await {
        Ok(v) => v,
        Err(e) => {
            error!("CHECK_FOR_UPDATE: failed to query local deno version {e}");
            return false;
        },
    };

    if current_deno_version != latest_deno_release.tag_name {
        info!("CHECK_FOR_UPDATE: deno is outdated");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        return true;
    }

    info!("CHECK_FOR_UPDATE: no update needed");
    
    false
}

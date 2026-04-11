use std::sync::Arc;

use futures_util::StreamExt;
use iced::task::Straw;
use iced::task::sipper;
use reqwest::Client;

use tokio::io::AsyncWriteExt;
use tracing::{info, error};

use crate::Paths;
use crate::VERSION;
use crate::github;
use crate::command::yt_dlp;
use crate::command::deno;

#[derive(Clone, Debug)]
pub struct UpdateResult {
    pub yt_dlp: bool,
    pub ffmpeg: bool,
    pub deno: bool,
    pub app: bool,
}

// Downloads all the needed assets based on what is missing
// Creates need directory structure if missing
pub fn download_assets(paths: Arc<Paths>, client: Client, is_update: bool) -> impl Straw<UpdateResult, DownloadProgress, ()> {
    sipper(async move |mut progress| {
        let mut result = UpdateResult {
            yt_dlp: false,
            ffmpeg: false,
            deno: false,
            app: false,
        };
      
        if !paths.downloader_dir.exists() {
            tokio::fs::create_dir(&paths.downloader_dir).await.map_err(|e|
                error!("DOWNLOAD_ASSETS: failed to create root YT Downloader directory -> {e}")
            )?;
        }

        if !paths.bin_dir.exists() {
            tokio::fs::create_dir(&paths.bin_dir).await.map_err(|e|
                error!("DOWNLOAD_ASSETS: failed to create bin directory -> {e}")
            )?;
        }

        if paths.tmp_dir.exists() {
            tokio::fs::remove_dir_all(&paths.tmp_dir).await.map_err(|e|
                error!("DOWNLOAD_ASSET: failed to delete old tmp directory -> {e}")
            )?;
        }

        if !paths.tmp_dir.exists() {
            tokio::fs::create_dir(&paths.tmp_dir).await.map_err(|e|
                error!("DOWNLOAD_ASSETS: failed to create tmp directory -> {e}")
            )?;
        }

        // Skip app update if installing for the first time
        if is_update {
            // Checks the update status of yt-dlp
            let latest_app_release = github::query_latest_release(
                client.clone(),
                github::APP_OWNER,
                github::APP_REPO,
            ).await.map_err(|e|
                error!("DOWNLOAD_ASSETS: failed to query latest app release -> {e}")
            )?;

            info!("DOWNLOAD_ASSETS: Latest app version: {}", latest_app_release.tag_name);

            let update_app = latest_app_release.tag_name != VERSION;

            if update_app {
                result.app = true;
                let mut file = tokio::fs::File::create(&paths.tmp_app_exe).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to create new yt_downloader.exe file -> {e}"))?;
            
                let mut asset_url = "";
                for e in &latest_app_release.assets {
                    if e.name == "yt_downloader.exe" {
                        asset_url = &e.url;
                    }
                }

                let response = github::start_asset_download(client.clone(), asset_url).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to intialize app download with the github api -> {e}"))?;

                let total = response.content_length().ok_or(())
                    .map_err(|_| error!("DOWNLOAD_ASSETS: failed to get app content length"))?;

                progress.send(DownloadProgress::Downloading(Asset::App, 0.0)).await;

                let mut byte_stream = response.bytes_stream();
                let mut downloaded = 0;

                info!("DOWNLOAD_ASSETS: downloading app: bytes={total} url={asset_url}");

                while let Some(next_bytes) = byte_stream.next().await {
                    let bytes = next_bytes.map_err(|e| info!("DOWNLOAD_ASSETS: failed to download app -> {e}"))?;
                    downloaded += bytes.len();
                    file.write_all(&bytes).await
                        .map_err(|e| error!("DOWNLOAD_ASSETS: failed to write to yt_downloader.exe file -> {e}"))?;

                    progress.send(DownloadProgress::Downloading(Asset::App, (downloaded as f32) / (total as f32))).await;
                }
            }
        }

        // Checks the update status of yt-dlp
        let latest_yt_dlp_release = github::query_latest_release(
            client.clone(),
            github::YT_DLP_OWNER,
            github::YT_DLP_REPO,
        ).await.map_err(|e|
            error!("DOWNLOAD_ASSETS: failed to query latest yt-dlp release -> {e}")
        )?;

        info!("DOWNLOAD_ASSETS: Latest yt-dlp version: {}", latest_yt_dlp_release.tag_name);

        let mut update_yt_dlp = true;
        if paths.yt_dlp_exe.exists() {
            progress.send(DownloadProgress::QueryingVersion(Asset::YtDlp)).await;

            let current_yt_dlp_version = yt_dlp::query_version(&paths.yt_dlp_exe).await
                .map_err(|e|
                    error!("DOWNLOAD_ASSETS: failed to query local yt-dlp -> {e}")
                )
                .unwrap_or("unknown".to_string());

            info!("DOWNLOAD_ASSETS: installed yt-dlp version: {current_yt_dlp_version}");
            
            if current_yt_dlp_version == latest_yt_dlp_release.tag_name {
                update_yt_dlp = false;
            }
        }

        if update_yt_dlp {
            result.yt_dlp = true;
            if paths.yt_dlp_exe.exists() {
                tokio::fs::remove_file(&paths.yt_dlp_exe).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to remove old yt-dlp file -> {e}"))?;
            }

            let mut file = tokio::fs::File::create(&paths.tmp_yt_dlp_exe).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to create new yt-dlp file -> {e}"))?;
            
            let mut asset_url = "";
            for e in &latest_yt_dlp_release.assets {
                if e.name == "yt-dlp.exe" {
                    asset_url = &e.url;
                }
            }

            let response = github::start_asset_download(client.clone(), asset_url).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to intialize yt-dlp download with the github api -> {e}"))?;

            let total = response.content_length().ok_or(())
                .map_err(|_| error!("DOWNLOAD_ASSETS: failed to get yt-dlp content length"))?;

            progress.send(DownloadProgress::Downloading(Asset::YtDlp, 0.0)).await;

            let mut byte_stream = response.bytes_stream();
            let mut downloaded = 0;

            info!("DOWNLOAD_ASSETS: downloading yt-dlp: bytes={total} url={asset_url}");

            while let Some(next_bytes) = byte_stream.next().await {
                let bytes = next_bytes.map_err(|e| info!("DOWNLOAD_ASSETS: failed to download yt-dlp -> {e}"))?;
                downloaded += bytes.len();
                file.write_all(&bytes).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to write to yt-dlp file -> {e}"))?;

                progress.send(DownloadProgress::Downloading(Asset::YtDlp, (downloaded as f32) / (total as f32))).await;
            }
        }

        // Download newest ffmpeg when ytdlp updates
        if update_yt_dlp || !paths.ffmpeg_dir.exists() {
            result.ffmpeg = true;
            let latest_ffmpeg_release = github::query_latest_release(
                client.clone(),
                github::FFMPEG_OWNER,
                github::FFMPEG_REPO,
            ).await.map_err(|e| info!("DOWNLOAD_ASSETS: failed to query ffmpeg version with the github api -> {e}"))?;
            
            if paths.ffmpeg_dir.exists() {
                tokio::fs::remove_dir_all(&paths.ffmpeg_dir).await
                    .map_err(|e| info!("DOWNLOAD_ASSETS: failed to delete current ffmpeg dir -> {e}"))?;
            }

            let mut zip_path = paths.tmp_dir.clone();
            zip_path.push("ffmpeg-master-latest-win64-gpl-shared.zip");

            { // file scope
                if zip_path.exists() {
                    tokio::fs::remove_file(&zip_path).await
                        .map_err(|e| info!("DOWNLOAD_ASSETS: failed to delete old ffmpeg zip -> {e}"))?;
                }

                let mut file = tokio::fs::File::create(&zip_path).await
                    .map_err(|e| info!("DOWNLOAD_ASSETS: failed to create ffmpeg zip -> {e}"))?;
            
                let mut asset_url = "";
                for e in &latest_ffmpeg_release.assets {
                    if e.name == "ffmpeg-master-latest-win64-gpl-shared.zip" {
                        asset_url = &e.url;
                    }
                }

                let response = github::start_asset_download(client.clone(), asset_url).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to initialize ffmpeg download -> {e}"))?;

                let total = response.content_length().ok_or(())
                    .map_err(|_| error!("DOWNLOAD_ASSETS: failed to get ffmpeg content length"))?;

                progress.send(DownloadProgress::Downloading(Asset::Ffmpeg, 0.0)).await;

                let mut byte_stream = response.bytes_stream();
                let mut downloaded = 0;

                info!("DOWNLOAD_ASSETS: downloading ffmpeg: bytes={total} url={asset_url}");

                while let Some(next_bytes) = byte_stream.next().await {
                    let bytes = next_bytes
                        .map_err(|e| info!("DOWNLOAD_ASSETS: failed to download ffmpeg -> {e}"))?;
                    downloaded += bytes.len();
                    file.write_all(&bytes).await
                        .map_err(|e| info!("DOWNLOAD_ASSETS: failed to write to the ffmpeg file -> {e}"))?;

                    progress.send(DownloadProgress::Downloading(Asset::Ffmpeg, (downloaded as f32) / (total as f32))).await;
                }
            }

            progress.send(DownloadProgress::Extracting(Asset::Ffmpeg)).await;

            // Long blocking ops should be run in a thread pool
            tokio::task::block_in_place(|| {
                // zip crate is synchronus so we gotta do it like this
                let file = std::fs::File::open(&zip_path)
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to open ffmpeg archive -> {e}"))?;

                let mut zip_archive = zip::ZipArchive::new(file)
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to create extractor for ffmpeg -> {e}"))?;

                info!("DOWNLOAD_ASSETS: extracting ffmpeg archive");

                zip_archive.extract_unwrapped_root_dir(
                    &paths.tmp_ffmpeg_dir,
                    zip::read::root_dir_common_filter,
                )
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to extract ffmpeg -> {e}"))?;

                Ok::<_, ()>(())
            })?;

            tokio::fs::remove_file(&zip_path).await
                .map_err(|e| info!("DOWNLOAD_ASSETS: failed to remove zip archive -> {e}"))?;
        }

        // Checks the update status of deno
        let latest_deno_release = github::query_latest_release(
            client.clone(),
            github::DENO_OWNER,
            github::DENO_REPO,
        ).await.map_err(|e| error!("DOWNLOAD_ASSETS: failed to query deno version from github -> {e}"))?;

        info!("DOWNLOAD_ASSETS: latest deno version: {}", latest_deno_release.tag_name);
       
        let mut update_deno = true;
        if paths.deno_exe.exists() {
            progress.send(DownloadProgress::QueryingVersion(Asset::Deno)).await;
            
            let current_deno_version = deno::query_version(&paths.deno_exe).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to query local deno version -> {e}"))
                .unwrap_or("unknown".to_string());
            info!("DOWNLOAD_ASSETS: installed deno version: {current_deno_version}");

            if current_deno_version == latest_deno_release.tag_name {
                update_deno = false;
            }
        }

        if update_deno {
            result.deno = true;
            if paths.deno_exe.exists() {
                tokio::fs::remove_file(&paths.deno_exe).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to remove old deno version -> {e}"))?;
            }

            let mut zip_path = paths.tmp_dir.clone();
            zip_path.push("deno-x86_64-pc-windows-msvc.zip");

            { // file scope
                if zip_path.exists() {
                    tokio::fs::remove_file(&zip_path).await
                        .map_err(|e| error!("DOWNLOAD_ASSETS: failed to delete old deno zip file -> {e}"))?;
                }
                
                let mut file = tokio::fs::File::create(&zip_path).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to create deno zip file -> {e}"))?;
            
                let mut asset_url = "";
                for e in &latest_deno_release.assets {
                    if e.name == "deno-x86_64-pc-windows-msvc.zip" {
                        asset_url = &e.url;
                    }
                }

                let response = github::start_asset_download(client.clone(), asset_url).await
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to intialize deno download -> {e}"))?;

                let total = response.content_length().ok_or(())
                    .map_err(|_| error!("DOWNLOAD_ASSETS: failed to get deno content length"))?;

                progress.send(DownloadProgress::Downloading(Asset::Deno, 0.0)).await;

                let mut byte_stream = response.bytes_stream();
                let mut downloaded = 0;

                info!("DOWNLOAD_ASSETS: downloading deno: bytes={total} url={asset_url}");

                while let Some(next_bytes) = byte_stream.next().await {
                    let bytes = next_bytes
                        .map_err(|e| error!("DOWNLOAD_ASSETS: deno download failed -> {e}"))?;

                    downloaded += bytes.len();

                    file.write_all(&bytes).await
                        .map_err(|e| error!("DOWNLOAD_ASSETS: failed to write to deno zip file -> {e}"))?;

                    progress.send(DownloadProgress::Downloading(Asset::Deno, (downloaded as f32) / (total as f32))).await;
                }
            }

            progress.send(DownloadProgress::Extracting(Asset::Deno)).await;

            tokio::task::block_in_place(|| {
                // zip crate is synchronus so we gotta do it like this
                let file = std::fs::File::open(&zip_path)
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to open deno zip archive -> {e}"))?;

                let mut zip_archive = zip::ZipArchive::new(file)
                    .map_err(|e| error!("DOWNLOAD_ASSETS: failed to create deno extractor -> {e}"))?;

                info!("DOWNLOAD_ASSETS: extracting deno archive");

                zip_archive.extract_unwrapped_root_dir(
                    &paths.tmp_dir,
                    zip::read::root_dir_common_filter,
                )
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to extract deno zip archive -> {e}"))?;

                Ok::<_, ()>(())
            })?;

            tokio::fs::remove_file(&zip_path).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to delete deno zip archive -> {e}"))?;
        }

        if paths.old_yt_downloader_exe.exists() {
            tokio::fs::remove_file(&paths.old_yt_downloader_exe).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to delete legacy YT Downloader -> {e}"))?;
            info!("DOWNLOAD_ASSETS: removed legacy YT Downloader");
        }

        if paths.old_yt_dlp_exe.exists() {
            tokio::fs::remove_file(&paths.old_yt_dlp_exe).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to delete legacy yt-dlp.exe -> {e}"))?;
            info!("DOWNLOAD_ASSETS: removed legacy yt-dlp");
        }

        if paths.old_ffmpeg_exe.exists() {
            tokio::fs::remove_file(&paths.old_ffmpeg_exe).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to delete legacy ffmpeg.exe -> {e}"))?;
            info!("DOWNLOAD_ASSETS: removed legacy ffmpeg");
        }

        if paths.old_deno_exe.exists() {
            tokio::fs::remove_file(&paths.old_deno_exe).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to delete legacy deno.exe -> {e}"))?;
            info!("DOWNLOAD_ASSETS: removed old deno");
        }

        if paths.old_version_file.exists() {
            tokio::fs::remove_file(&paths.old_version_file).await
                .map_err(|e| error!("DOWNLOAD_ASSETS: failed to delete legacy version file -> {e}"))?;
            info!("DOWNLOAD_ASSETS: removed legacy version file");
        }

        Ok(result)
    })
}

#[derive(Clone, Debug)]
pub enum DownloadProgress {
    Downloading(Asset, f32),
    Extracting(Asset),
    QueryingVersion(Asset),
}

#[derive(Default, Debug, Clone)]
pub enum Asset {
    #[default]
    YtDlp,
    Ffmpeg,
    Deno,
    App,
}

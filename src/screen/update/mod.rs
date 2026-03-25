use std::sync::Arc;

use futures_util::StreamExt;
use iced::alignment::Horizontal;
use iced::task::Straw;
use iced::task::sipper;
use iced::{Task, Element};
use iced::widget::*;
use reqwest::Client;
use thiserror::Error;

use tokio::io::AsyncWriteExt;
use tracing::{info, error};

use crate::Paths;
use crate::github;
use crate::lang::Translation;
use crate::command::yt_dlp;
use crate::command::deno;
use crate::platform::windows::{
    error_dialog,
    finish_update_process,
    finish_install_process,
};
use crate::widget::circular::Circular;

#[derive(Clone, Default)]
pub enum UpdateKind {
    #[default]
    Install, // First time install
    Normal, // Normal update
}

#[derive(Default)]
pub struct Screen {
    paths: Arc<Paths>,

    kind: UpdateKind,

    current_asset: Asset,
    progess: f32,
    spinner: bool,
}

// I think downloading one asset at a time is better for most cases
impl Screen {
    // Unlike App screens can have state passed into them so we don't use the default trait
    pub fn new(
        paths: Arc<Paths>,
    ) -> Self {
        Self {
            paths,
            kind: UpdateKind::Normal,
            progess: 0.0,
            current_asset: Asset::YtDlp,
            spinner: true,
        }
    }

    pub fn start(
        &mut self,
        kind: UpdateKind,
        client: &Client,
    ) -> Task<Message> {
        self.kind = kind;

        Task::sip(
            download_assets(
                Arc::clone(&self.paths),
                client.clone(),
            ),
            Message::Working,
            Message::Finished,
        )
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> Action {
        match message {
            Message::ScriptLaunched(r) => {
                let _ = r.inspect_err(|e| {
                    use crate::platform::windows::Error as Er;
                    match e {
                        Er::GetUserPreferredLanguageFailed(ie) => error!("{e} {ie}"),
                        Er::ConvertOsStringToUTF8Failed => error!("{e}"),
                        Er::GetExePathFailed(ie) => error!("{e} {ie}"),
                        Er::SpawnPowershellCommandFailed(ie) => error!("{e} {ie}"),
                        Er::OpenRegistryKeyFailed(ie) => error!("{e} {ie}"),
                        Er::QueryThemeFailed(ie) => error!("{e} {ie}"),
                    }

                    match self.kind {
                        UpdateKind::Install => error_dialog("Failed to launch install script"),
                        UpdateKind::Normal => error_dialog("Failed to launch update script"),
                    }
                });

                info!("Update applied app relaunching");

                Action::Exit
            },
            
            Message::Working(p) => {
                match p {
                    DownloadProgress::Downloading(asset, progress) => {
                        self.current_asset = asset;
                        self.progess = progress;
                        self.spinner = false;
                    },

                    DownloadProgress::QueryingVersion(asset) => {
                        self.current_asset = asset;
                        self.spinner = true;
                    }

                    DownloadProgress::Extracting(asset) => {
                        self.current_asset = asset;
                        self.spinner = true;
                    }
                }

                Action::None
            },

            Message::Finished(e) => {
                match e {
                    Ok(_) => {
                        Action::Done(Some(match self.kind {
                            UpdateKind::Install =>
                                Task::perform(finish_install_process(), Message::ScriptLaunched),
                            UpdateKind::Normal =>
                                Task::perform(finish_update_process(), Message::ScriptLaunched),
                        }))
                    },

                    Err(e) => {
                        match &e {
                            DownloadError::WriteDenoZipFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DownloadDenoFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::InitializeDenoDownloadFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::CreateDenoZipFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DeleteDenoFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DeleteFfmpegZipFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::ExtractFfmpegFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::OpenFfmpegZipFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::WriteFfmpegZipFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::InitializeFfmpegDownloadFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DownloadFfmpegFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::CreateFfmpegZipFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DeleteFfmpegDirFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::QueryFfmpegLatestReleaseFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::WriteYtDlpFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::CreateYtDlpFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DeleteYtDlpFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DownloaderDirectoryCreationFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::BinDirectoryCreationFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::QueryYtDlpLatestReleaseFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::QueryLocalYtDlpVersionFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::InitializeYtDlpDownloadFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DownloadYtDlpFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::QueryDenoLatestReleaseFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::QueryLocalDenoVersionFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::OpenDenoZipFileFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::ExtractDenoFailed(ie) => error!("{e} -> {ie}"),
                            DownloadError::DeleteOldYtDownloader(ie) => error!("{e} -> {ie}"),
                            _ => error!("{e}"),
                        }

                        match self.kind {
                            UpdateKind::Install => {
                                error_dialog("Installation failed");
                            },
                            UpdateKind::Normal => {
                                error_dialog("Update failed");
                            },
                        }

                        Action::Done(None)
                    },
                }
            },
        }
    }
    
    pub fn view(&self, translation: &Translation) -> Element<'_, Message> {
        let label = match self.kind {
            UpdateKind::Install => translation.update_screen_caption_install,
            UpdateKind::Normal => translation.update_screen_caption_update,
        };

        let asset = match self.current_asset {
            Asset::YtDlp => "1/3   yt-dlp",
            Asset::Ffmpeg => "2/3   ffmpeg",
            Asset::Deno => "3/3   deno",
        };
        
        center(
            Column::new()
                .push(text(label).size(30))
                .push(space().height(30))
                .push(
                    if self.spinner {
                        // What is this hell? Why can't rust just infer this...
                        <Circular<'_, Theme> as Into<Element<'_, Message>>>::into(Circular::new())
                    } else {
                        progress_bar(0.0f32..=1.0f32, self.progess).into()
                    }
                )
                .push(space().height(30))
                .push(text(asset))
                .align_x(Horizontal::Center)
                .padding(30)
        )
        .into()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Working(DownloadProgress),
    Finished(Result<(), DownloadError>),
    ScriptLaunched(crate::platform::windows::Result<()>),
}

pub enum Action {
    None,
    Exit,
    Done(Option<Task<Message>>),
}

// Downloads all the needed assets based on what is missing
// Creates need directory structure if missing
fn download_assets(paths: Arc<Paths>, client: Client) -> impl Straw<(), DownloadProgress, DownloadError> {
    sipper(async move |mut progress| {
        if !paths.downloader_dir.exists() {
            tokio::fs::create_dir(&paths.downloader_dir).await.map_err(|e|
                DownloadError::DownloaderDirectoryCreationFailed(Arc::new(e)),
            )?;
        }

        if !paths.bin_dir.exists() {
            tokio::fs::create_dir(&paths.bin_dir).await.map_err(|e|
                DownloadError::BinDirectoryCreationFailed(Arc::new(e)),
            )?;
        }

        // Checks the update status of yt-dlp
        let latest_yt_dlp_release = github::query_latest_release(
            client.clone(),
            github::YT_DLP_OWNER,
            github::YT_DLP_REPO,
        ).await.map_err(DownloadError::QueryYtDlpLatestReleaseFailed)?;
        info!("Latest yt-dlp version: {}", latest_yt_dlp_release.tag_name);

        let mut update_yt_dlp = true;
        if paths.yt_dlp_exe.exists() {
            progress.send(DownloadProgress::QueryingVersion(Asset::YtDlp)).await;

            let current_yt_dlp_version = yt_dlp::query_version(&paths.yt_dlp_exe).await
                .map_err(DownloadError::QueryLocalYtDlpVersionFailed)?;
            info!("Installed yt-dlp version: {current_yt_dlp_version}");
            
            if current_yt_dlp_version == latest_yt_dlp_release.tag_name {
                update_yt_dlp = false;
            }
        }

        if update_yt_dlp {
            if paths.yt_dlp_exe.exists() {
                tokio::fs::remove_file(&paths.yt_dlp_exe).await
                    .map_err(|e| DownloadError::DeleteYtDlpFileFailed(Arc::new(e)))?;
            }

            let mut file = tokio::fs::File::create(&paths.yt_dlp_exe).await
                .map_err(|e| DownloadError::CreateYtDlpFileFailed(Arc::new(e)))?;
            
            let mut asset_url = "";
            for e in &latest_yt_dlp_release.assets {
                if e.name == "yt-dlp.exe" {
                    asset_url = &e.url;
                }
            }

            let response = github::start_asset_download(client.clone(), asset_url).await
                .map_err(DownloadError::InitializeYtDlpDownloadFailed)?;

            let total = response.content_length().ok_or(DownloadError::NoContentLength)?;
            progress.send(DownloadProgress::Downloading(Asset::YtDlp, 0.0)).await;

            let mut byte_stream = response.bytes_stream();
            let mut downloaded = 0;

            info!("Downloading yt-dlp: bytes={total} url={asset_url}");

            while let Some(next_bytes) = byte_stream.next().await {
                let bytes = next_bytes.map_err(|e| DownloadError::DownloadYtDlpFailed(Arc::new(e)))?;
                downloaded += bytes.len();
                file.write_all(&bytes).await
                    .map_err(|e| DownloadError::WriteYtDlpFileFailed(Arc::new(e)))?;

                progress.send(DownloadProgress::Downloading(Asset::YtDlp, (downloaded as f32) / (total as f32))).await;
            }
        }

        // Download newest ffmpeg when ytdlp updates
        if update_yt_dlp || !paths.ffmpeg_dir.exists() {
            let latest_ffmpeg_release = github::query_latest_release(
                client.clone(),
                github::FFMPEG_OWNER,
                github::FFMPEG_REPO,
            ).await.map_err(DownloadError::QueryFfmpegLatestReleaseFailed)?;
            
            if paths.ffmpeg_dir.exists() {
                tokio::fs::remove_dir_all(&paths.ffmpeg_dir).await
                    .map_err(|e| DownloadError::DeleteFfmpegDirFailed(Arc::new(e)))?;
            }

            let mut zip_path = paths.bin_dir.clone();
            zip_path.push("ffmpeg-master-latest-win64-gpl-shared.zip");

            { // file scope
                if zip_path.exists() {
                    tokio::fs::remove_file(&zip_path).await
                        .map_err(|e| DownloadError::DeleteFfmpegZipFileFailed(Arc::new(e)))?;
                }

                let mut file = tokio::fs::File::create(&zip_path).await
                    .map_err(|e| DownloadError::CreateFfmpegZipFileFailed(Arc::new(e)))?;
            
                let mut asset_url = "";
                for e in &latest_ffmpeg_release.assets {
                    if e.name == "ffmpeg-master-latest-win64-gpl-shared.zip" {
                        asset_url = &e.url;
                    }
                }

                let response = github::start_asset_download(client.clone(), asset_url).await
                    .map_err(DownloadError::InitializeFfmpegDownloadFailed)?;

                let total = response.content_length().ok_or(DownloadError::NoContentLength)?;
                progress.send(DownloadProgress::Downloading(Asset::Ffmpeg, 0.0)).await;

                let mut byte_stream = response.bytes_stream();
                let mut downloaded = 0;

                info!("Downloading ffmpeg: bytes={total} url={asset_url}");

                while let Some(next_bytes) = byte_stream.next().await {
                    let bytes = next_bytes.map_err(|e| DownloadError::DownloadFfmpegFailed(Arc::new(e)))?;
                    downloaded += bytes.len();
                    file.write_all(&bytes).await
                        .map_err(|e| DownloadError::WriteFfmpegZipFileFailed(Arc::new(e)))?;

                    progress.send(DownloadProgress::Downloading(Asset::Ffmpeg, (downloaded as f32) / (total as f32))).await;
                }
            }

            progress.send(DownloadProgress::Extracting(Asset::Ffmpeg)).await;

            // Long blocking ops should be run in a thread pool
            tokio::task::block_in_place(|| {
                // zip crate is synchronus so we gotta do it like this
                let file = std::fs::File::open(&zip_path)
                    .map_err(|e| DownloadError::OpenFfmpegZipFileFailed(Arc::new(e)))?;

                let mut zip_archive = zip::ZipArchive::new(file)
                    .map_err(|e| DownloadError::ExtractFfmpegFailed(Arc::new(e)))?;

                info!("Extracting ffmpeg archive");

                zip_archive.extract_unwrapped_root_dir(
                    &paths.ffmpeg_dir,
                    zip::read::root_dir_common_filter,
                )
                .map_err(|e| DownloadError::ExtractFfmpegFailed(Arc::new(e)))?;

                Ok::<_, DownloadError>(())
            })?;

            tokio::fs::remove_file(&zip_path).await
                .map_err(|e| DownloadError::DeleteFfmpegZipFileFailed(Arc::new(e)))?;
        }

        // Checks the update status of deno
        let latest_deno_release = github::query_latest_release(
            client.clone(),
            github::DENO_OWNER,
            github::DENO_REPO,
        ).await.map_err(DownloadError::QueryDenoLatestReleaseFailed)?;
        info!("Latest deno version: {}", latest_deno_release.tag_name);
       
        let mut update_deno = true;
        if paths.deno_exe.exists() {
            progress.send(DownloadProgress::QueryingVersion(Asset::Deno)).await;
            
            let current_deno_version = deno::query_version(&paths.deno_exe).await
                .map_err(DownloadError::QueryLocalDenoVersionFailed)?;
            info!("Installed deno version: {current_deno_version}");

            if current_deno_version == latest_deno_release.tag_name {
                update_deno = false;
            }
        }

        if update_deno {
            if paths.deno_exe.exists() {
                tokio::fs::remove_file(&paths.deno_exe).await
                    .map_err(|e| DownloadError::DeleteDenoFileFailed(Arc::new(e)))?;
            }

            let mut zip_path = paths.bin_dir.clone();
            zip_path.push("deno-x86_64-pc-windows-msvc.zip");

            { // file scope
                if zip_path.exists() {
                    tokio::fs::remove_file(&zip_path).await
                        .map_err(|e| DownloadError::DeleteDenoZipFileFailed(Arc::new(e)))?;
                }
                
                let mut file = tokio::fs::File::create(&zip_path).await
                    .map_err(|e| DownloadError::CreateDenoZipFileFailed(Arc::new(e)))?;
            
                let mut asset_url = "";
                for e in &latest_deno_release.assets {
                    if e.name == "deno-x86_64-pc-windows-msvc.zip" {
                        asset_url = &e.url;
                    }
                }

                let response = github::start_asset_download(client.clone(), asset_url).await
                    .map_err(DownloadError::InitializeDenoDownloadFailed)?;

                let total = response.content_length().ok_or(DownloadError::NoContentLength)?;
                progress.send(DownloadProgress::Downloading(Asset::Deno, 0.0)).await;

                let mut byte_stream = response.bytes_stream();
                let mut downloaded = 0;

                info!("Downloading deno: bytes={total} url={asset_url}");

                while let Some(next_bytes) = byte_stream.next().await {
                    let bytes = next_bytes.map_err(|e| DownloadError::DownloadDenoFailed(Arc::new(e)))?;
                    downloaded += bytes.len();
                    file.write_all(&bytes).await
                        .map_err(|e| DownloadError::WriteDenoZipFileFailed(Arc::new(e)))?;

                    progress.send(DownloadProgress::Downloading(Asset::Deno, (downloaded as f32) / (total as f32))).await;
                }
            }

            progress.send(DownloadProgress::Extracting(Asset::Deno)).await;

            tokio::task::block_in_place(|| {
                // zip crate is synchronus so we gotta do it like this
                let file = std::fs::File::open(&zip_path)
                    .map_err(|e| DownloadError::OpenDenoZipFileFailed(Arc::new(e)))?;

                let mut zip_archive = zip::ZipArchive::new(file)
                    .map_err(|e| DownloadError::ExtractDenoFailed(Arc::new(e)))?;

                info!("Extracting deno archive");

                zip_archive.extract_unwrapped_root_dir(
                    &paths.bin_dir,
                    zip::read::root_dir_common_filter,
                )
                .map_err(|e| DownloadError::ExtractDenoFailed(Arc::new(e)))?;

                Ok::<_, DownloadError>(())
            })?;

            tokio::fs::remove_file(&zip_path).await
                .map_err(|e| DownloadError::DeleteDenoZipFileFailed(Arc::new(e)))?;
        }
         
        if paths.old_yt_downloader_exe.exists() {
            tokio::fs::remove_file(&paths.old_yt_downloader_exe).await
                .map_err(|e| DownloadError::DeleteOldYtDownloader(Arc::new(e)))?;
            info!("Removed old YT Downloader");
        }

        if paths.old_yt_dlp_exe.exists() {
            tokio::fs::remove_file(&paths.old_yt_dlp_exe).await
                .map_err(|e| DownloadError::DeleteOldYtDownloader(Arc::new(e)))?;
            info!("Removed old yt-dlp");
        }

        if paths.old_ffmpeg_exe.exists() {
            tokio::fs::remove_file(&paths.old_ffmpeg_exe).await
                .map_err(|e| DownloadError::DeleteOldYtDownloader(Arc::new(e)))?;
            info!("Removed old ffmpeg");
        }

        if paths.old_deno_exe.exists() {
            tokio::fs::remove_file(&paths.old_deno_exe).await
                .map_err(|e| DownloadError::DeleteOldYtDownloader(Arc::new(e)))?;
            info!("Removed old deno");
        }

        if paths.old_version_file.exists() {
            tokio::fs::remove_file(&paths.old_version_file).await
                .map_err(|e| DownloadError::DeleteOldYtDownloader(Arc::new(e)))?;
            info!("Removed old version file");
        }

        Ok(())
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
}

#[derive(Error, Debug, Clone)]
pub enum DownloadError {
    #[error("failed to delete old YT Downloader installation")]
    DeleteOldYtDownloader(Arc<std::io::Error>),
    
    #[error("failed to delete deno zip file")]
    DeleteDenoZipFileFailed(Arc<std::io::Error>),

    #[error("failed to extract deno")]
    ExtractDenoFailed(Arc<zip::result::ZipError>),

    #[error("failed to open deno zip file")]
    OpenDenoZipFileFailed(Arc<std::io::Error>),

    #[error("failed to write bytes to write to deno zip file")]
    WriteDenoZipFileFailed(Arc<std::io::Error>),

    #[error("failed to download deno")]
    DownloadDenoFailed(Arc<reqwest::Error>),

    #[error("failed to initialize deno download")]
    InitializeDenoDownloadFailed(github::Error),

    #[error("failed to create deno zip file")]
    CreateDenoZipFileFailed(Arc<std::io::Error>),

    #[error("failed to delete \"%LocalAppData%/YT Downloader/bin/deno.exe\"")]
    DeleteDenoFileFailed(Arc<std::io::Error>),

    #[error("failed to delete ffmpeg zip file")]
    DeleteFfmpegZipFileFailed(Arc<std::io::Error>),

    #[error("failed to extract ffmpeg")]
    ExtractFfmpegFailed(Arc<zip::result::ZipError>),
    
    #[error("failed to open ffmpeg zip file")]
    OpenFfmpegZipFileFailed(Arc<std::io::Error>),

    #[error("failed to write bytes to write to ffmpeg zip file")]
    WriteFfmpegZipFileFailed(Arc<std::io::Error>),

    #[error("failed to initialize ffmpeg download")]
    InitializeFfmpegDownloadFailed(github::Error),

    #[error("failed to download ffmpeg")]
    DownloadFfmpegFailed(Arc<reqwest::Error>),

    #[error("failed to create ffmpeg zip file")]
    CreateFfmpegZipFileFailed(Arc<std::io::Error>),

    #[error("failed to delete \"%LocalAppData%/YT Downloader/bin/ffmpeg\"")]
    DeleteFfmpegDirFailed(Arc<std::io::Error>),

    #[error("failed to get the latest release for ffmpeg from the github api")]
    QueryFfmpegLatestReleaseFailed(github::Error),

    #[error("failed to write bytes to \"%LocalAppData%/YT Downloader/bin/yt-dlp.exe\" file")]
    WriteYtDlpFileFailed(Arc<std::io::Error>),

    #[error("failed to create \"%LocalAppData%/YT Downloader/bin/yt-dlp.exe\" file")]
    CreateYtDlpFileFailed(Arc<std::io::Error>),
    
    #[error("failed to delete \"%LocalAppData%/YT Downloader/bin/yt-dlp.exe\"")]
    DeleteYtDlpFileFailed(Arc<std::io::Error>),
    
    #[error("failed to create \"%LocalAppData%/YT Downloader\" directory")]
    DownloaderDirectoryCreationFailed(Arc<std::io::Error>),

    #[error("failed to create \"%LocalAppData%/YT Downloader/bin\" directory")]
    BinDirectoryCreationFailed(Arc<std::io::Error>),

    #[error("failed to get the latest release for yt-dlp from the github api")]
    QueryYtDlpLatestReleaseFailed(github::Error),

    #[error("failed to get the installed yt-dlp version")]
    QueryLocalYtDlpVersionFailed(yt_dlp::Error),

    #[error("failed to initialize yt-dlp download")]
    InitializeYtDlpDownloadFailed(github::Error),

    #[error("failed to download yt-dlp")]
    DownloadYtDlpFailed(Arc<reqwest::Error>),

    #[error("failed to get the latest release for deno from the github api")]
    QueryDenoLatestReleaseFailed(github::Error),
    
    #[error("failed to get the installed deno version")]
    QueryLocalDenoVersionFailed(deno::Error),
    
    #[error("no content length")]
    NoContentLength,
}

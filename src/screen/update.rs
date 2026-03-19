use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::task::Straw;
use iced::task::sipper;
use iced::{Task, Element};
use iced::widget::*;
use reqwest::Client;
use thiserror::Error;

use tracing::info;

use crate::Paths;
use crate::github;
use crate::lang::Translation;
use crate::command::yt_dlp;
use crate::command::deno;

#[derive(Clone)]
pub enum UpdateKind {
    Install, // First time install
    Normal, // Normal update
}

pub struct Screen {
    paths: Arc<Paths>,

    kind: UpdateKind,
    progess: f32,
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
            Message::Working(DownloadProgress::QueryingVersion(Asset::YtDlp)) => {
                tracing::info!("Querying version of yt-dlp");
                Action::None
            },
            _ => Action::None,
        }
    }
    
    pub fn view(&self, translation: &Translation) -> Element<'_, Message> {
        let label = match self.kind {
            UpdateKind::Install => translation.update_screen_install_label,
            UpdateKind::Normal => translation.update_screen_update_label,
        };
        
        center(
            Column::new()
                .push(text(label).size(30))
                .push(space().height(30))
                .push(progress_bar(0.0f32..=1.0f32, self.progess))
                .align_x(Horizontal::Center)
                .padding(30)
        )
        .into()
    }
}

#[derive(Clone)]
pub enum Message {
    Working(DownloadProgress),
    Finished(Result<(), DownloadError>),
}

pub enum Action {
    None,
    Run(Task<Message>),
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

        let mut update_deno = true;
        if paths.deno_exe.exists() {
            progress.send(DownloadProgress::QueryingVersion(Asset::Deno)).await;
            
            let current_deno_version = deno::query_version(&paths.deno_exe).await
                .map_err(DownloadError::QueryLocalDenoVersionFailed)?;
            info!("Installed deno version: {current_deno_version}");
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

#[derive(Debug, Clone)]
pub enum Asset {
    YtDlp,
    Ffmpeg,
    Deno,
}

#[derive(Error, Debug, Clone)]
pub enum DownloadError {
    #[error("failed to create \"%LocalAppData%/YT Downloader\" directory")]
    DownloaderDirectoryCreationFailed(Arc<std::io::Error>),

    #[error("failed to create \"%LocalAppData%/YT Downloader/bin\" directory")]
    BinDirectoryCreationFailed(Arc<std::io::Error>),

    #[error("failed to get the latest release for yt-dlp from the github api")]
    QueryYtDlpLatestReleaseFailed(github::Error),

    #[error("failed to get the installed yt-dlp version")]
    QueryLocalYtDlpVersionFailed(yt_dlp::Error),

    #[error("failed to get the installed deno version")]
    QueryLocalDenoVersionFailed(deno::Error),
}

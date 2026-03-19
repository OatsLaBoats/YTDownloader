use std::sync::Arc;

use thiserror::Error;
use iced::alignment::Horizontal;
use iced::task::Straw;
use iced::task::sipper;
use iced::*;
use iced::widget::*;

use reqwest::Client;

use crate::Paths;
use crate::github;
use crate::lang::Translation;

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
        &self,
        kind: UpdateKind,
        client: &Client,
    ) -> Task<Message> {
        Task::sip(
            download_assets(
                Arc::clone(&self.paths),
                client.clone(),
            ),
            Message::Working,
           |x| {
               Message::Finished
           },
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
    Finished,
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
            tokio::fs::create_dir(&paths.downloader_dir).await.unwrap();
        }

        if !paths.bin_dir.exists() {
            tokio::fs::create_dir(&paths.bin_dir).await.unwrap();
        }

        let latest_yt_dlp_release = github::get_latest_yt_dlp_release(client.clone()).await?;
        tracing::info!("Latest yt-dlp version: {}", latest_yt_dlp_release.tag_name);

        let mut update_yt_dlp = true;
        if !paths.yt_dlp_exe.exists() {
            progress.send(DownloadProgress::QueryingVersion(Asset::YtDlp)).await;

            let result = tokio::process::Command::new("yt-dlp")
                .kill_on_drop(true)
                .arg("--version")
                .output().await;

            let result = match result {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("yt-dlp not found {e}");
                    return Err(DownloadError::Other);
                },
            }.stdout;

            let current_version = str::from_utf8(&result).unwrap();
            tracing::info!("Installed yt-dlp version: {current_version}");

            if current_version.contains(&latest_yt_dlp_release.tag_name) {
                tracing::info!("No need for update");
                update_yt_dlp = false;
            }
        }
        
        Ok(())
    })
}

#[derive(Clone, Debug)]
enum DownloadProgress {
    Downloading(Asset, f32),
    Extracting(Asset),
    QueryingVersion(Asset),
}

#[derive(Debug, Clone)]
enum Asset {
    YtDlp,
    Ffmpeg,
    Deno,
}

#[derive(Debug, Clone)]
enum DownloadError {
    AnyhowError(Arc<anyhow::Error>),
    Other,
}

impl From<anyhow::Error> for DownloadError {
    fn from(value: anyhow::Error) -> Self {
        DownloadError::AnyhowError(Arc::new(value))
    }
}

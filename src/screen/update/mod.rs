use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::{Task, Element};
use iced::widget::*;
use reqwest::Client;

use tracing::{info, error};

use crate::Paths;
use crate::lang::Translation;
use crate::platform::windows::{
    error_dialog,
    finish_update_process,
    finish_install_process,
};
use crate::widget::circular::Circular;

mod tasks;
use tasks::*;

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
                        Er::ConvertRustStringToCStringFailed => error!("{e}"),
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
                    Ok(r) => {
                        Action::Done(Some(match self.kind {
                            UpdateKind::Install => {
                                info!("Install finished");
                                Task::perform(finish_install_process(
                                    r.yt_dlp,
                                    r.ffmpeg,
                                    r.deno,
                                ), Message::ScriptLaunched)
                            },
                            UpdateKind::Normal => {
                                info!("Update finished");
                                Task::perform(finish_update_process(
                                    r.yt_dlp,
                                    r.ffmpeg,
                                    r.deno,
                                    r.app,
                                ), Message::ScriptLaunched)
                            },
                        }))
                    },

                    Err(_) => {
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
            Asset::App => "1/4   app",
            Asset::YtDlp => "2/4   yt-dlp",
            Asset::Ffmpeg => "3/4   ffmpeg",
            Asset::Deno => "4/4   deno",
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
    Finished(Result<UpdateResult, ()>),
    ScriptLaunched(crate::platform::windows::Result<()>),
}

pub enum Action {
    None,
    Exit,
    Done(Option<Task<Message>>),
}

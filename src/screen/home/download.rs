use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::Arc;
use std::time::Duration;

use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Task, color, never};
use iced::widget::*;
use iced::widget::column;
use tracing::{info, error};

use crate::command::yt_dlp::{AudioDownloadParams, AudioFileType, AudioInfo, DownloadParams, DownloadProgress, VideoDownloadParams, VideoFileType, VideoInfo, VideoQuality, download_media};
use crate::lang::Translation;
use crate::platform::windows::kill_process;
use crate::screen::TOOLTIP_DELAY;
use crate::{AudioConversionQuality, Images, Paths, SponsorBlockOption};
use crate::widget::linear::Linear;

#[derive(Debug, Clone)]
pub enum DownloadInfo {
    Video(VideoDownloadInfo),
    Audio(AudioDownloadInfo),
}

#[derive(Debug, Clone)]
pub struct AudioDownloadInfo {
    pub info: AudioInfo,
    pub conversion_quality: AudioConversionQuality,
    pub selected_format: AudioFileType,
    pub sb_options: Option<SponsorBlockOption>,
    pub download_location: String,
    pub link: String,
    pub force_ipv4: bool,
}

#[derive(Debug, Clone)]
pub struct VideoDownloadInfo {
    pub info: VideoInfo,
    pub selected_format: VideoFileType,
    pub selected_quality: VideoQuality,
    pub sb_options: Option<SponsorBlockOption>,
    pub download_location: String,
    pub link: String,
    pub force_ipv4: bool,
}

#[derive(Debug, Clone)]
pub enum MessageKind {
    Working(DownloadProgress),
    Finished(std::result::Result<(), ()>),
    Cleanup(()),
    Pause,
    Close,
    OpenFolder,
    Debug,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: usize,
    pub kind: MessageKind,
}

impl Message {
    pub fn new(id: usize, kind: MessageKind) -> Self {
        Self {
            id,
            kind,
        }
    }

    pub fn debug(id: usize) -> Self {
        Self::new(id, MessageKind::Debug)
    }

    pub fn pause(id: usize) -> Self {
        Self::new(id, MessageKind::Pause)
    }

    pub fn close(id: usize) -> Self {
        Self::new(id, MessageKind::Close)
    }

    pub fn open_folder(id: usize) -> Self {
        Self::new(id, MessageKind::OpenFolder)
    }

    pub fn cleanup(id: usize) -> impl FnMut(()) -> Message {
        move |p| Self::new(id, MessageKind::Cleanup(p))
    }

    pub fn working(id: usize) -> impl FnMut(DownloadProgress) -> Message {
        move |progress| Self::new(id, MessageKind::Working(progress))
    }

    pub fn finished(id: usize) -> impl FnMut(std::result::Result<(), ()>) -> Message {
        move |result| Self::new(id, MessageKind::Finished(result))
    }
}

pub enum ActionKind {
    None,
    Close,
    Run(Task<Message>),
    RunClose(Task<Message>),
}

pub struct Action {
    #[allow(unused)]
    pub id: usize,
    pub kind: ActionKind,
}

impl Action {
    pub fn new(id: usize, kind: ActionKind) -> Self {
        Self {
            id,
            kind,
        }
    }

    pub fn none(id: usize) -> Self {
        Self::new(id, ActionKind::None)
    }

    pub fn close(id: usize) -> Self {
        Self::new(id, ActionKind::Close)
    }

    pub fn run(id: usize, task: Task<Message>) -> Self {
        Self::new(id, ActionKind::Run(task))
    }

    pub fn run_close(id: usize, task: Task<Message>) -> Self {
        Self::new(id, ActionKind::RunClose(task))
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
enum ProgressState {
    Starting,
    Downloading,
    PostProcessing,
    Finished(bool),
}

#[derive(Hash)]
struct VideoId<'a> {
    url: &'a str,
    id: usize,
}

impl<'a> VideoId<'a> {
    fn gen_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

pub struct State {
    id: usize,
    paths: Arc<Paths>,
    info: DownloadInfo,
    hash: u64,

    paused: bool,
    progress_state: ProgressState,
    task: Option<iced::task::Handle>,
    process: Option<u32>,

    title: Option<String>,
    total_bytes: usize,
    total_bytes_estimate: f64,
    downloaded_bytes: usize,
    eta: u32,
    download_speed: usize,
    percent: f64,
}

impl State {
    pub fn new(id: usize, paths: Arc<Paths>, info: DownloadInfo) -> Self {
        let title = match &info {
            DownloadInfo::Video(v) => {
                v.info.title.clone()
            },

            DownloadInfo::Audio(v) => {
                v.info.title.clone()
            },
        };

        Self {
            id,
            paths,
            info,
            hash: id as u64,

            paused: false,
            progress_state: ProgressState::Starting,
            task: None,
            process: None,

            title,
            total_bytes: 0,
            downloaded_bytes: 0,
            eta: 0,
            download_speed: 0,
            total_bytes_estimate: 0.0,
            percent: 0.0,
        }
    }

    pub fn start(&mut self) -> Task<Message> {
        let output_path;
        let link_;

        let params;
        let force_ipv4;

        match &self.info {
            DownloadInfo::Video(vi) => {
                output_path = &vi.download_location;
                link_ = vi.link.clone();
                force_ipv4 = vi.force_ipv4;

                params = DownloadParams::Video(VideoDownloadParams {
                    quality: vi.selected_quality.clone(),
                    format: vi.selected_format.clone(),
                    sb_options: vi.sb_options.clone(),
                });
            },

            DownloadInfo::Audio(ai) => {
                output_path = &ai.download_location;
                link_ = ai.link.clone();
                force_ipv4 = ai.force_ipv4;

                params = DownloadParams::Audio(AudioDownloadParams {
                    quality: ai.conversion_quality.clone(),
                    format: ai.selected_format.clone(),
                    sb_options: ai.sb_options.clone(),
                });
            },
        }

        let vi = VideoId {
            url: &link_,
            id: self.id,
        };

        self.hash = vi.gen_hash();
        
        let sip = download_media(
            self.hash,
            self.paths.video_dir.clone(),
            self.paths.yt_dlp_exe.clone(),
            self.paths.ffmpeg_bin_dir.clone(),
            self.paths.deno_exe.clone(),
            output_path.clone(),
            link_,
            force_ipv4,
            params,
        );

        let (task, handle) = Task::sip(
            sip,
            Message::working(self.id),
            Message::finished(self.id),
        )
        .abortable();

        self.task = Some(handle);

        task
    }

    pub fn update(&mut self, message: MessageKind) -> Action {
        match message {
            MessageKind::Cleanup(()) => {
                Action::close(self.id)
            },
            
            MessageKind::OpenFolder => {
                let dir = match &self.info {
                    DownloadInfo::Video(v) => {
                        v.download_location.clone()
                    },

                    DownloadInfo::Audio(v) => {
                        v.download_location.clone()
                    },
                };

                match crate::platform::windows::open_file_explorer(&dir) {
                    Ok(_) => info!("opened file explorer"),
                    Err(e) => error!("failed to open file explorer -> {e}"),
                }

                Action::none(self.id)
            },

            MessageKind::Close => {
                info!("stopping download");
                if let ProgressState::Finished(_) = self.progress_state {
                    return Action::close(self.id);
                }

                if let Some(h) = &self.task && !h.is_aborted() {
                    h.abort();
                }

                // Ensure it is killed even if tokio doesn't
                if let Some(id) = self.process {
                    let _ = kill_process(id);
                }

                let mut download_path = self.paths.video_dir.clone();
                download_path.push(format!("dl{}", self.hash));

                let task = Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        let _ = tokio::fs::remove_dir_all(&download_path).await
                            .map_err(|e| error!("failed to clean up {download_path:?} -> {e}"));
                    },

                    Message::cleanup(self.id),
                );
               
                Action::run_close(self.id, task)
            },

            MessageKind::Pause => {
                self.paused = !self.paused;

                if self.paused {
                    info!("download paused");
                    if let Some(h) = &self.task && !h.is_aborted() {
                        h.abort();
                    }

                    Action::none(self.id)
                } else {
                    info!("download resumed");
                    let task = self.start();
                    Action::run(self.id, task)
                }
            },

            MessageKind::Working(progress) => {
                match progress {
                    DownloadProgress::Starting(pid) => {
                        self.process = pid;
                        self.progress_state = ProgressState::Starting;
                    },
        
                    DownloadProgress::Downloading(p) => {
                        if self.title.is_none() {
                            self.title = p.title;
                        }

                        if let Some(v) = p.total_bytes {
                            self.total_bytes = v;
                        }

                        if let Some(v) = p.downloaded_bytes {
                            self.downloaded_bytes = v;
                        }

                        if let Some(v) = p.eta {
                            self.eta = v;
                        }

                        if let Some(v) = p.download_speed {
                            self.download_speed = v;
                        }

                        if let Some(v) = p.total_bytes_estimate {
                            self.total_bytes_estimate = v;
                        }

                        if let Some(v) = p.percent {
                            self.percent = v;
                        }

                        if self.percent == 100.0 {
                            self.progress_state = ProgressState::PostProcessing;
                        } else {
                            self.progress_state = ProgressState::Downloading;
                        }
                    },
                }

                Action::none(self.id)
            },

            MessageKind::Finished(result) => {
                info!("finished download with state {result:?}");
                self.progress_state = ProgressState::Finished(result.is_err());
                Action::none(self.id)
            },

            MessageKind::Debug => Action::none(self.id),
        }
    }

    pub fn view(&self, translation: &Translation, images: &Images) -> Element<'_, Message> {
        let eta_minutes = self.eta / 60;
        let eta_seconds = self.eta - eta_minutes;

        let invalid_percent =
            self.percent >= 100.0 ||
            self.percent <= 0.0;

        let speed: Element<'_, Message> = if self.download_speed != 0 {
            text(format!("{:.2} MB/s", self.download_speed as f64 / 1024.0 / 1024.0)).into()
        } else {
            space().into()
        };

        let bar: Element<'_, Message> = if self.progress_state == ProgressState::Downloading && !invalid_percent {
            stack![
                container(progress_bar(0.0f32..=100.0f32, self.percent as f32))
                    .height(20),
                center(speed),
            ]
            .into()
        } else if let ProgressState::Finished(_) = self.progress_state {
            space().into()
        } else {
            Linear::new()
                .into()
        };

        let status: Element<'_, Message> = match self.progress_state {
            ProgressState::Starting => text(translation.download_status_starting).into(),
            ProgressState::Downloading => text(translation.download_status_downloading).into(),
            ProgressState::PostProcessing => text(translation.download_status_postprocessing).into(),
            ProgressState::Finished(error) => if error {
                rich_text![
                    span(translation.download_status_failed)
                        .color(color!(0xff0000)),
                ]
                .on_link_click(never)
                .into()
            } else {
                text(translation.download_status_finished).into()
            },
        };

        let title: &str = match &self.title {
            Some(v) => v,
            None => translation.general_unknown,
        };

        let close_button: Element<'_, Message> = if self.progress_state == ProgressState::PostProcessing {
            space().into()
        } else {
            tooltip(
                button(
                    center(image(images.close.clone()))
                        .width(20)
                        .height(20),
                )
                .on_press(Message::close(self.id)),
                container(translation.tooltip_download_close_desc)
                    .padding(10)
                    .style(container::rounded_box),
                tooltip::Position::Bottom,
            )
            .delay(TOOLTIP_DELAY)
            .into()
        };

        let action_button: Element<'_, Message> =
            if self.progress_state == ProgressState::PostProcessing {
                space().into()
            } else if let ProgressState::Finished(_) = self.progress_state {
                tooltip(
                    button(
                        center(image(images.folder.clone()))
                            .width(20)
                            .height(20),
                    )
                    .on_press(Message::open_folder(self.id)),
                    container(translation.tooltip_download_open_desc)
                        .padding(10)
                        .style(container::rounded_box),
                    tooltip::Position::Bottom,
                )
                .delay(TOOLTIP_DELAY)
                .into()
            } else if self.paused {
                button(
                    center(image(images.play.clone()))
                        .width(20)
                        .height(20),
                )
                .on_press(Message::pause(self.id))
                .into()
            } else {
                button(
                    center(image(images.pause.clone()))
                        .width(20)
                        .height(20),
                )
                .on_press(Message::pause(self.id))
                .into()
            };

        let eta: Element<'_, Message> = if let ProgressState::Downloading = self.progress_state {
            text(format!("{:02}:{:02}", eta_minutes, eta_seconds)).into()
        } else {
            space().into()
        };
       
        center(
            column![
                row![
                    text(title),
                ]
                .align_y(Vertical::Center),

                row![
                    space().width(10),
                    row![
                        bar,
                        space().width(5),
                        eta,
                    ]
                    .align_y(Vertical::Center),
                    space().width(10),
                ],

                row![
                    status,
                ]
                .align_y(Vertical::Center),

                row![
                    action_button,
                    close_button,
                ]
                .spacing(5)
                .align_y(Vertical::Center),

                space().height(5),
                rule::horizontal(1),
            ]
            .spacing(5)
            .align_x(Horizontal::Center),
        )
        .into()
    }
}

use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Length, Task, color, never};
use iced::widget::*;
use iced::widget::column;
use tracing::{error, info};

use crate::screen::home::download::DownloadInfo;
use crate::widget::circular::Circular;
use crate::{Paths, Settings};
use crate::command::yt_dlp::{AudioFileType, AudioInfo, LinkInfo, PlaylistInfo, PlaylistItem, VideoFileType, VideoInfo, VideoQuality};
use crate::command::yt_dlp;
use crate::lang::Translation;

// TODO: Force ipv4 when failing

enum LinkError {
    InvalidUrl,
    InfoRetrievalFailed,
}

pub struct State {
    paths: Arc<Paths>,
    settings: Settings,

    loading_link_info: bool,
    link_info: Option<LinkInfo>,
    link_error: Option<LinkError>,
    selected_video_quality: VideoQuality,

    playlist_items: combo_box::State<Arc<PlaylistItem>>,
    selected_playlist_item: Option<Arc<PlaylistItem>>,
    playlist_link_info: Option<LinkInfo>,
    loading_playlist_link_info: bool,

    task_handle: Option<iced::task::Handle>,
    retry: bool,
    link: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    LinkInfoQueryFinished(yt_dlp::Result<LinkInfo>),

    VideoQualitySelected(VideoQuality),
    VideoFormatSelected(VideoFileType),

    AudioFormatSelected(AudioFileType),
    AudioOnlyToggled(bool),

    PlaylistItemSelected(Arc<PlaylistItem>),
    PlaylistLinkInfoQueryFinished(yt_dlp::Result<LinkInfo>),

    Download(DownloadInfo),
}

pub enum Action {
    None,
    Run(Task<Message>),
    SettingsChanged(Settings),
    Download(DownloadInfo),
}

impl State {
    pub fn new(
        settings: Settings,
        paths: Arc<Paths>,
    ) -> Self {
        Self {
            paths,
            settings,
            loading_link_info: false,
            link_info: None,
            link_error: None,
            selected_video_quality: VideoQuality::Best,
            playlist_items: combo_box::State::new(Vec::new()),
            selected_playlist_item: None,
            playlist_link_info: None,
            loading_playlist_link_info: false,
            task_handle: None,
            retry: true,
            link: String::new(),
        }
    }

    pub fn start(&mut self, link: String) -> Option<Task<Message>> {
        if let Some(th) = &mut self.task_handle && !th.is_aborted() {
            th.abort();
        }

        self.task_handle = None;
        self.loading_link_info = false;
        self.link_info = None;
        self.link_error = None;
        self.playlist_items = combo_box::State::new(Vec::new());
        self.selected_playlist_item = None;
        self.playlist_link_info = None;
        self.loading_playlist_link_info = false;
        self.retry = true;
        self.link = link.clone();

        match url::Url::parse(&link) {
            Ok(_) => {
                info!("querying link info");
                self.loading_link_info = true;

                let (task, handle) = Task::perform(
                    yt_dlp::query_link_info(
                        false,
                        self.paths.yt_dlp_exe.clone(),
                        self.paths.ffmpeg_bin_dir.clone(),
                        self.paths.deno_exe.clone(),
                        link,
                    ),
                    Message::LinkInfoQueryFinished,
                ).abortable();

                self.task_handle = Some(handle);
                Some(task)
            },

            Err(e) => {
                error!("invalid url input -> {e}");
                self.link_error = Some(LinkError::InvalidUrl);
                None
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Download(i) => {
                Action::Download(i)
            },
            
            Message::VideoQualitySelected(q) => {
                self.selected_video_quality = q;
                Action::None
            },

            Message::AudioOnlyToggled(b) => {
                self.settings.audio_only = b;
                Action::SettingsChanged(self.settings.clone())
            },

            Message::VideoFormatSelected(f) => {
                self.settings.video_format = f;
                Action::SettingsChanged(self.settings.clone())
            },

            Message::AudioFormatSelected(f) => {
                self.settings.audio_format = f;
                Action::SettingsChanged(self.settings.clone())
            }

            Message::PlaylistItemSelected(i) => {
                info!("querying playlist link info");
                self.loading_playlist_link_info = true;
                self.link_error = None;

                self.selected_playlist_item = Some(Arc::clone(&i));
                self.retry = true;

                if let Some(th) = &mut self.task_handle && !th.is_aborted() {
                    th.abort();
                }

                let (task, handle) = Task::perform(
                    yt_dlp::query_link_info(
                        false,
                        self.paths.yt_dlp_exe.clone(),
                        self.paths.ffmpeg_dir.clone(),
                        self.paths.deno_exe.clone(),
                        i.url.clone(),
                    ),
                    Message::PlaylistLinkInfoQueryFinished,
                ).abortable();

                self.task_handle = Some(handle);
                Action::Run(task)
            },

            Message::PlaylistLinkInfoQueryFinished(r) => {
                self.loading_playlist_link_info = false;

                match r {
                    Ok(v) => {
                        info!("playlist link info retrieved successfully");

                        match &v {
                            LinkInfo::Video(i) => {
                                if i.f_1080.is_some() {
                                    self.selected_video_quality = VideoQuality::Q1080;
                                } else {
                                    self.selected_video_quality = VideoQuality::Best;
                                }

                                self.playlist_link_info = Some(v);
                            },

                            LinkInfo::Playlist(_) => {
                                error!("playlist can not contain an other playlist");
                                self.playlist_link_info = None;
                            },

                            _ => self.playlist_link_info = Some(v),
                        }

                        Action::None
                    },

                    Err(e) => {
                        if self.retry {
                            error!("failed to retrieve link info -> {e}");
                            info!("retry querying link info via ipv4");
                            self.loading_playlist_link_info = true;
                            self.retry = false;

                            if let Some(th) = &mut self.task_handle && !th.is_aborted() {
                                th.abort();
                            }

                            let (task, handle) = Task::perform(
                                yt_dlp::query_link_info(
                                    true,
                                    self.paths.yt_dlp_exe.clone(),
                                    self.paths.ffmpeg_dir.clone(),
                                    self.paths.deno_exe.clone(),
                                    self.selected_playlist_item.as_ref().unwrap().url.clone(),
                                ),
                                Message::PlaylistLinkInfoQueryFinished,
                            ).abortable();

                            self.task_handle = Some(handle);
                            Action::Run(task)
                        } else {
                            error!("failed to retrieve link info -> {e}");
                            self.link_info = None;
                            self.link_error = Some(LinkError::InfoRetrievalFailed);
                            Action::None
                        }
                    },
                }
            },

            Message::LinkInfoQueryFinished(r) => {
                self.loading_link_info = false;

                match r {
                    Ok(v) => {
                        info!("link info retrieved successfully");

                        match &v {
                            LinkInfo::Video(i) => {
                                if i.f_1080.is_some() {
                                    self.selected_video_quality = VideoQuality::Q1080;
                                } else {
                                    self.selected_video_quality = VideoQuality::Best;
                                }
                            },

                            LinkInfo::Playlist(i) => {
                                self.playlist_items = combo_box::State::new(i.items.clone());
                            },

                            _ => {},
                        }
                        
                        self.link_info = Some(v);
                        Action::None
                    },

                    Err(e) => {
                        if self.retry {
                            error!("failed to retrieve link info -> {e}");
                            info!("retry querying link info via ipv4");
                            self.loading_link_info = true;
                            self.retry = false;

                            if let Some(th) = &mut self.task_handle && !th.is_aborted() {
                                th.abort();
                            }

                            let (task, handle) = Task::perform(
                                yt_dlp::query_link_info(
                                    true,
                                    self.paths.yt_dlp_exe.clone(),
                                    self.paths.ffmpeg_dir.clone(),
                                    self.paths.deno_exe.clone(),
                                    self.link.clone(),
                                ),
                                Message::PlaylistLinkInfoQueryFinished,
                            ).abortable();

                            self.task_handle = Some(handle);
                            Action::Run(task)
                        } else {
                            error!("failed to retrieve link info -> {e}");
                            self.link_info = None;
                            self.link_error = Some(LinkError::InfoRetrievalFailed);
                            Action::None
                        }
                    },
                }
            },
        }
    }

    pub fn view<'a>(&'a self, translation: &'a Translation) -> Element<'a, Message> {
        self.ipanel(translation, self.link_info.as_ref(), self.loading_link_info)
    }

    fn ipanel<'a>(
        &'a self,
        translation: &'a Translation,
        link_info: Option<&'a LinkInfo>,
        loading: bool,
    ) -> Element<'a, Message> {
        let panel: Element<'a, Message> = if let Some(e) = &self.link_error {
            match e {
                LinkError::InvalidUrl => {
                    rich_text![
                        span("Invalid url\nMake sure the link is correct")
                            .color(color!(0xff0000)),
                    ]
                    .on_link_click(never)
                    .center()
                    .into()
                },

                LinkError::InfoRetrievalFailed => {
                    rich_text![
                        span("Failed to retrieve link information\nMake sure the link refers to valid media")
                            .color(color!(0xff0000)),
                    ]
                    .on_link_click(never)
                    .center()
                    .into()
                },
            }
        } else if loading {
            let s = if self.retry {
                "Loading link..."
            } else {
                "Retrying..."
            };

            column![
                Circular::new(),
                space().height(30),
                text(s),
            ]
            .align_x(Horizontal::Center)
            .into()
        } else if let Some(li) = &link_info {
            match li {
                LinkInfo::Playlist(playlist) => self.playlist_panel(translation, playlist),
                LinkInfo::Video(video) => self.video_info_panel(translation, video),
                LinkInfo::Audio(audio) => self.audio_info_panel(translation, audio),
            }
        } else {
            space().into()
        };

        panel.into()
    }

    fn playlist_panel<'a>(
        &'a self,
        translation: &'a Translation,
        info: &'a PlaylistInfo,
    ) -> Element<'a, Message> {
        let selection: Element<'a, Message> = combo_box(
            &self.playlist_items,
            "Select playlist item...",
            self.selected_playlist_item.as_ref(),
            Message::PlaylistItemSelected,
        )
        .menu_height(200)
        .into();

        let ppanel = self.ipanel(
            translation,
            if let Some(LinkInfo::Playlist(_)) = self.playlist_link_info {
                None
            } else {
               self.playlist_link_info.as_ref()
            },
            self.loading_playlist_link_info,
        );

        row![
            column![
                text(format!("{}", info.name)),
                row![
                    space().width(50),
                    selection,
                    space().width(50),
                ]
                .align_y(Vertical::Center),

                space().height(30),
                ppanel,
            ]
            .align_x(Horizontal::Center)
        ]
        .into()
    }

    fn audio_info_panel<'a>(
        &'a self,
        translation: &'a Translation,
        info: &'a AudioInfo,
    ) -> Element<'a, Message> {
        let title = match &info.title {
            Some(v) => v,
            None => "Unknown",
        };

        let channel = match &info.channel {
            Some(v) => v,
            None => "Unknown",
        };

        let options: Element<'a, Message> =
            row![
                text("Format"),
                space().width(5),
                pick_list(
                    AudioFileType::file_types(),
                    Some(&self.settings.audio_format),
                    Message::AudioFormatSelected,
                ),
            ]
            .align_y(Vertical::Center)
            .into();

        row![
            column![
                text(format!("{}", title)).size(20),
                text(format!("{} {}", "by", channel)),
                space().height(30),
                options,
                space().height(30),
                button("Download")
                    .on_press(
                        Message::Download(
                            DownloadInfo::Audio {
                                info: info.clone(),
                                selected_format: self.settings.audio_format.clone(),
                                extract_from_video: false,
                            },
                        ),
                    ),
            ]
            .align_x(Horizontal::Center)
        ]
        .into()
    }

    fn video_info_panel<'a>(
        &'a self,
        translation: &'a Translation,
        info: &'a VideoInfo,
    ) -> Element<'a, Message> {
        let title = match &info.title {
            Some(v) => v,
            None => "Unknown",
        };

        let channel = match &info.channel {
            Some(v) => v,
            None => "Unknown",
        };

        let options: Element<'a, Message> = if !self.settings.audio_only {
            row![
                text("Quality"),
                space().width(5),
                pick_list(
                    info.qualities(),
                    Some(&self.selected_video_quality),
                    Message::VideoQualitySelected,
                ),

                space().width(50),

                text("Format"),
                space().width(5),
                pick_list(
                    VideoFileType::file_types(),
                    Some(&self.settings.video_format),
                    Message::VideoFormatSelected,
                ),
            ]
            .align_y(Vertical::Center)
            .into()
        } else {
            row![
                text("Format"),
                space().width(5),
                pick_list(
                    AudioFileType::file_types(),
                    Some(&self.settings.audio_format),
                    Message::AudioFormatSelected,
                ),
            ]
            .align_y(Vertical::Center)
            .into()
        };

        row![
            column![
                text(format!("{}", title)).size(20),
                text(format!("{} {}", "by", channel)),

                space().height(30),

                checkbox(self.settings.audio_only)
                    .label("Audio only")
                    .on_toggle(Message::AudioOnlyToggled),
            
                space().height(30),
                options,
                space().height(30),
                button("Download")
                    .on_press(
                        Message::Download(
                            if self.settings.audio_only {
                                DownloadInfo::Audio {
                                    info: info.to_audio_info(),
                                    selected_format: self.settings.audio_format.clone(),
                                    extract_from_video: true,
                                }
                            } else {
                                DownloadInfo::Video {
                                    info: info.clone(),
                                    selected_format: self.settings.video_format.clone(),
                                    selected_quality: self.selected_video_quality.clone(),
                                }
                            },
                        ),
                    ),
            ]
            .align_x(Horizontal::Center)
        ]
        .into()
    }
}

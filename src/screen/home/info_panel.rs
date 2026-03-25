use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Task, color, never};
use iced::widget::*;
use iced::widget::column;
use tracing::{error, info};

use crate::screen::home::download::DownloadInfo;
use crate::screen::home::tasks::open_file_picker;
use crate::widget::circular::Circular;
use crate::{AudioConversionQuality, Images, Paths, Settings};
use crate::command::yt_dlp::{AudioFileType, AudioInfo, LinkInfo, PlaylistInfo, PlaylistItem, VideoFileType, VideoInfo, VideoQuality};
use crate::command::yt_dlp;
use crate::lang::Translation;

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
    download_link: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    UpdateSettings(Settings),
    
    LinkInfoQueryFinished(yt_dlp::Result<LinkInfo>),

    SponsorBlockToggled(bool),
    SbSponsorToggled(bool),
    SbIntroToggled(bool),
    SbOutroToggled(bool),
    SbSelfpromoToggled(bool),
    SbPreviewToggled(bool),
    SbFillerToggled(bool),
    SbInteractionToggled(bool),
    SbMusicOfftopicToggled(bool),
    SbHookToggled(bool),
    SbChapterToggled(bool),
    SbAllToggled(bool),

    VideoQualitySelected(VideoQuality),
    VideoFormatSelected(VideoFileType),
    RemuxToggled(bool),

    AudioConversionQualitySelected(AudioConversionQuality),
    AudioFormatSelected(AudioFileType),
    AudioOnlyToggled(bool),

    PlaylistItemSelected(Arc<PlaylistItem>),
    PlaylistLinkInfoQueryFinished(yt_dlp::Result<LinkInfo>),

    OpenFilePicker,
    TargetLocationChanged(Option<String>),

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
            download_link: String::new(),
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
            Message::UpdateSettings(settings) => {
                self.settings = settings;
                Action::None
            },
            
            Message::OpenFilePicker => {
                Action::Run(
                    Task::perform(
                        open_file_picker(self.settings.download_dir.clone()),
                        Message::TargetLocationChanged,
                    ),
                )
            },

            Message::TargetLocationChanged(s) => {
                match s {
                    None => Action::None,
                    Some(s) => {
                        self.settings.download_dir = s;
                        Action::SettingsChanged(self.settings.clone())
                    },
                }
            },
            
            Message::SbChapterToggled(b) => {
                self.settings.sb_options.sb_chapter = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbHookToggled(b) => {
                self.settings.sb_options.sb_hook = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbMusicOfftopicToggled(b) => {
                self.settings.sb_options.sb_music_offtopic = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbInteractionToggled(b) => {
                self.settings.sb_options.sb_interaction = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbFillerToggled(b) => {
                self.settings.sb_options.sb_filler = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbPreviewToggled(b) => {
                self.settings.sb_options.sb_preview = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbSelfpromoToggled(b) => {
                self.settings.sb_options.sb_selfpromo = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbOutroToggled(b) => {
                self.settings.sb_options.sb_outro = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbIntroToggled(b) => {
                self.settings.sb_options.sb_intro = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbSponsorToggled(b) => {
                self.settings.sb_options.sb_sponsor = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SbAllToggled(b) => {
                self.settings.sb_options.sb_all = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::SponsorBlockToggled(b) => {
                self.settings.sponsor_block = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::AudioConversionQualitySelected(acq) => {
                self.settings.conversion_quality = acq;
                Action::SettingsChanged(self.settings.clone())
            },
            
            Message::RemuxToggled(b) => {
                self.settings.remux = b;
                Action::SettingsChanged(self.settings.clone())
            },
            
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

                self.download_link.clear();
                self.download_link.push_str(&i.url);

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
                        self.download_link.clear();
                        self.download_link.push_str(&self.link);

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

    pub fn view<'a>(&'a self, translation: &'a Translation, images: &'a Images) -> Element<'a, Message> {
        self.ipanel(translation, images, self.link_info.as_ref(), self.loading_link_info)
    }

    fn ipanel<'a>(
        &'a self,
        translation: &'a Translation,
        images: &'a Images,
        link_info: Option<&'a LinkInfo>,
        loading: bool,
    ) -> Element<'a, Message> {
        let panel: Element<'a, Message> = if let Some(e) = &self.link_error {
            match e {
                LinkError::InvalidUrl => {
                    rich_text![
                        span(translation.info_panel_link_error)
                            .color(color!(0xff0000)),
                    ]
                    .size(20)
                    .on_link_click(never)
                    .center()
                    .into()
                },

                LinkError::InfoRetrievalFailed => {
                    rich_text![
                        span(translation.info_panel_media_error)
                            .color(color!(0xff0000)),
                    ]
                    .size(20)
                    .on_link_click(never)
                    .center()
                    .into()
                },
            }
        } else if loading {
            let s = if self.retry {
                translation.info_panel_loading_message_attemp1_label
            } else {
                translation.info_panel_loading_message_attemp2_label
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
                LinkInfo::Playlist(playlist) => self.playlist_panel(translation, images, playlist),
                LinkInfo::Video(video) => self.video_info_panel(translation, images, video),
                LinkInfo::Audio(audio) => self.audio_info_panel(translation, images, audio),
            }
        } else {
            space().into()
        };

        panel.into()
    }

    fn playlist_panel<'a>(
        &'a self,
        translation: &'a Translation,
        images: &'a Images,
        info: &'a PlaylistInfo,
    ) -> Element<'a, Message> {
        let selection: Element<'a, Message> = combo_box(
            &self.playlist_items,
            translation.info_panel_playlist_item_placeholder,
            self.selected_playlist_item.as_ref(),
            Message::PlaylistItemSelected,
        )
        .menu_height(200)
        .into();

        let ppanel = self.ipanel(
            translation,
            images,
            if let Some(LinkInfo::Playlist(_)) = self.playlist_link_info {
                None
            } else {
               self.playlist_link_info.as_ref()
            },
            self.loading_playlist_link_info,
        );

        row![
            column![
                text(&info.name),
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
        images: &'a Images,
        info: &'a AudioInfo,
    ) -> Element<'a, Message> {
        let title = match &info.title {
            Some(v) => v,
            None => translation.general_unknown,
        };

        let channel = match &info.channel {
            Some(v) => v,
            None => translation.general_unknown,
        };

        let cb_sponsor_block: Element<'a, Message> =
            checkbox(self.settings.sponsor_block)
                .label("SponsorBlock")
                .on_toggle(Message::SponsorBlockToggled)
                .into();

        let sponsor_block_options: Element<'a, Message> = if self.settings.sponsor_block {
            self.sponsor_block_options()
        } else {
            space().into()
        };

        let options: Element<'a, Message> =
            row![
                text(translation.general_quality),
                space().width(5),
                pick_list(
                    [AudioConversionQuality::High, AudioConversionQuality::Medium, AudioConversionQuality::Low],
                    Some(&self.settings.conversion_quality),
                    Message::AudioConversionQualitySelected,
                ),

                space().width(50),

                text(translation.general_format),
                space().width(5),
                pick_list(
                    AudioFileType::file_types(),
                    Some(&self.settings.audio_format),
                    Message::AudioFormatSelected,
                ),
            ]
            .align_y(Vertical::Center)
            .into();

        let sb_options = if self.settings.sponsor_block {
            Some(self.settings.sb_options.clone())
        } else {
            None
        };

        column![
            text(title).size(20),
            text(format!("{} {}", translation.general_by, channel)),
            space().height(30),

            cb_sponsor_block,

            sponsor_block_options,
            space().height(30),
            
            options,

            space().height(30),
            text(translation.info_panel_download_location_label),
            space().height(5),
            row![
                text_input("", &self.settings.download_dir)
                    .style(download_path_style),
                space().width(5),
                
                button(
                    center(image(images.folder.clone()))
                        .width(30)
                        .height(21),
                )
                .on_press(Message::OpenFilePicker),
            ]
            .align_y(Vertical::Center),

            space().height(30),
            button(
                row![
                    text(translation.general_download),
                    space().width(5),
                    center(image(images.download.clone()))
                        .width(30)
                        .height(30),
                ]
                .align_y(Vertical::Center),
            )
            .on_press(
                Message::Download(
                    DownloadInfo::Audio {
                        info: info.clone(),
                        conversion_quality: self.settings.conversion_quality,
                        selected_format: self.settings.audio_format.clone(),
                        extract_from_video: false,
                        remux: false,
                        sb_options,
                        download_location: self.settings.download_dir.clone(),
                        link: self.download_link.clone(),
                    },
                ),
            ),
        ]
        .align_x(Horizontal::Center)
        .into()
    }

    fn video_info_panel<'a>(
        &'a self,
        translation: &'a Translation,
        images: &'a Images,
        info: &'a VideoInfo,
    ) -> Element<'a, Message> {
        let title = match &info.title {
            Some(v) => v,
            None => translation.general_unknown,
        };

        let channel = match &info.channel {
            Some(v) => v,
            None => translation.general_unknown,
        };

        let cb_audio_only: Element<'a, Message> = if info.has_audio {
            checkbox(self.settings.audio_only)
                .label(translation.info_panel_audio_only_checkbox)
                .on_toggle(Message::AudioOnlyToggled)
                .into()
        } else {
            space().into()
        };

        let cb_remux: Element<'a, Message> =
            checkbox(self.settings.remux)
                .label("Remux")
                .on_toggle(Message::RemuxToggled)
                .into();

        let cb_sponsor_block: Element<'a, Message> =
            checkbox(self.settings.sponsor_block)
                .label("SponsorBlock")
                .on_toggle(Message::SponsorBlockToggled)
                .into();

        let sponsor_block_options: Element<'a, Message> = if self.settings.sponsor_block {
            self.sponsor_block_options()
        } else {
            space().into()
        };

        let options: Element<'a, Message> = if !self.settings.audio_only {
            row![
                text(translation.general_quality),
                space().width(5),
                pick_list(
                    info.qualities(),
                    Some(&self.selected_video_quality),
                    Message::VideoQualitySelected,
                ),

                space().width(50),

                text(translation.general_format),
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
                text(translation.general_quality),
                space().width(5),
                pick_list(
                    [AudioConversionQuality::High, AudioConversionQuality::Medium, AudioConversionQuality::Low],
                    Some(&self.settings.conversion_quality),
                    Message::AudioConversionQualitySelected,
                ),

                space().width(50),

                text(translation.general_format),
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

        let sb_options = if self.settings.sponsor_block {
            Some(self.settings.sb_options.clone())
        } else {
            None
        };

        column![
            text(title).size(20),
            text(format!("{} {}", translation.general_by, channel)),

            space().height(30),

            row![
                cb_audio_only,
                space().width(20),
                cb_remux,
                space().width(20),
                cb_sponsor_block,
            ]
            .align_y(Vertical::Center),

            sponsor_block_options,
        
            space().height(30),
            options,

            space().height(30),
            text(translation.info_panel_download_location_label),
            space().height(5),
            row![
                text_input("", &self.settings.download_dir)
                    .style(download_path_style),
                space().width(5),
                
                button(
                    center(image(images.folder.clone()))
                        .width(30)
                        .height(21),
                )
                .on_press(Message::OpenFilePicker),
            ]
            .align_y(Vertical::Center),

            space().height(30),
            button(
                row![
                    text(translation.general_download),
                    space().width(5),
                    center(image(images.download.clone()))
                        .width(30)
                        .height(30),
                ]
                .align_y(Vertical::Center),
            )
            .on_press(
                Message::Download(
                    if self.settings.audio_only && info.has_audio {
                        DownloadInfo::Audio {
                            info: info.to_audio_info(),
                            conversion_quality: self.settings.conversion_quality,
                            selected_format: self.settings.audio_format.clone(),
                            extract_from_video: true,
                            remux: self.settings.remux,
                            sb_options,
                            download_location: self.settings.download_dir.clone(),
                            link: self.download_link.clone(),
                        }
                    } else {
                        DownloadInfo::Video {
                            info: info.clone(),
                            selected_format: self.settings.video_format.clone(),
                            selected_quality: self.selected_video_quality.clone(),
                            remux: self.settings.remux,
                            sb_options,
                            download_location: self.settings.download_dir.clone(),
                            link: self.download_link.clone(),
                        }
                    },
                ),
            ),
        ]
        .align_x(Horizontal::Center)
        .into()
    }

    pub fn sponsor_block_options<'a>(&'a self) -> Element<'a, Message> {
        column![
            space().height(30),
            row![
                column![
                    checkbox(self.settings.sb_options.sb_all)
                        .label("all")
                        .on_toggle(Message::SbAllToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_sponsor)
                        .label("sponsor")
                        .on_toggle(Message::SbSponsorToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_intro)
                        .label("intro")
                        .on_toggle(Message::SbIntroToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_hook)
                        .label("hook")
                        .on_toggle(Message::SbHookToggled),
                ],

                space().width(20),

                column![
                    checkbox(self.settings.sb_options.sb_outro)
                        .label("outro")
                        .on_toggle(Message::SbOutroToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_selfpromo)
                        .label("self promo")
                        .on_toggle(Message::SbSelfpromoToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_preview)
                        .label("preview")
                        .on_toggle(Message::SbPreviewToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_chapter)
                        .label("chapter")
                        .on_toggle(Message::SbChapterToggled),
                ],

                space().width(20),

                column![
                    checkbox(self.settings.sb_options.sb_filler)
                        .label("filler")
                        .on_toggle(Message::SbFillerToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_interaction)
                        .label("interaction")
                        .on_toggle(Message::SbInteractionToggled),

                    space().width(20),

                    checkbox(self.settings.sb_options.sb_music_offtopic)
                        .label("music offtopic")
                        .on_toggle(Message::SbMusicOfftopicToggled),
                ],
            ],
        ]
        .align_x(Horizontal::Center)
        .into()
    }
}

fn download_path_style(theme: &iced::Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        ..text_input::default(theme, text_input::Status::Active)
    }
}

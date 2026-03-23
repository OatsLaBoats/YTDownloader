use std::sync::Arc;

use iced::{Color, Length, Padding, color, never};
use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Task};
use iced::widget::*;
use iced::widget::column;
use iced_aw::menu::Item;
use iced_aw::*;
use reqwest::Client;
use tracing::{info, error};

use crate::command::yt_dlp::{AudioFileType, AudioInfo, PlaylistInfo, PlaylistItem, VideoFileType, VideoInfo, VideoQuality};
use crate::command::{deno, yt_dlp};
use yt_dlp::LinkInfo;
use crate::platform::windows::{error_dialog, uninstall};
use crate::widget::circular::Circular;
use crate::{Images, Paths, Settings, github};
use crate::lang::Translation;
use super::download::{Download};

// TODO: Pause, and cancel download buttons
// You can query progress using _percent, eta, tmpfilename
// Query video info using templaes instead of json because it seem too error prone
//
// TODO: Hide download list when nothing is downloading
// TODO: Make link info cancelable

// TODO: save some things in settings
// TODO: Refactor

pub struct Screen {
    paths: Arc<Paths>,
    settings: Settings,

    show_update_popup: bool,
    show_credits_popup: bool,
    show_uninstall_popup: bool,

    link_input: String,
    loading_link_info: bool,
    link_info: Option<LinkInfo>,
    link_error: Option<LinkError>,
    selected_video_quality: VideoQuality,
    selected_video_format: VideoFileType,
    selected_audio_format: AudioFileType,
    audio_only: bool,

    playlist_items: combo_box::State<Arc<PlaylistItem>>,
    selected_playlist_item: Option<Arc<PlaylistItem>>,
    playlist_link_info: Option<LinkInfo>,
    loading_playlist_link_info: bool,
}

enum LinkError {
    InvalidUrl,
    InfoRetrievalFailed,
}

#[derive(Clone, Debug)]
pub enum Message {
    CheckForUpdate(bool),
    UpdateNow,
    UpdateLater,
    LinkInputChanged(String),
    ThemeSelected(crate::Theme),
    LanguageSelected(crate::lang::Language),
    AutoUpdatesToggled(bool),
    ShowCreditsPopup(bool),
    ShowUninstallPopup(bool),
    Uninstall,
    UninstallScriptLaunched(crate::platform::windows::Result<()>),
    PasteLink,
    ClipboardRead(Option<String>),
    LinkInfoQueryFinished(yt_dlp::Result<LinkInfo>),
    VideoQualitySelected(VideoQuality),
    VideoFormatSelected(VideoFileType),
    AudioFormatSelected(AudioFileType),
    AudioOnlyToggled(bool),
    PlaylistItemSelected(Arc<PlaylistItem>),
    PlaylistLinkInfoQueryFinished(yt_dlp::Result<LinkInfo>),
    Debug,
}

pub enum Action {
    None,
    UpdateNeeded,
    SettingsChanged(Settings),
    Run(Task<Message>),
    Exit,
}

impl Screen {
    pub fn new(paths: Arc<Paths>, settings: Settings) -> Self {
        Self {
            paths,
            settings,
            show_update_popup: false,
            show_credits_popup: false,
            show_uninstall_popup: false,
            link_input: String::new(),
            loading_link_info: false,
            link_info: None,
            link_error: None,
            selected_video_quality: VideoQuality::Best,
            selected_video_format: VideoFileType::MP4,
            selected_audio_format: AudioFileType::MP3,
            audio_only: false,
            playlist_items: combo_box::State::new(Vec::new()),
            selected_playlist_item: None,
            playlist_link_info: None,
            loading_playlist_link_info: false,
        }
    }

    pub fn start(&mut self, client: &Client) -> Task<Message> {
        if self.settings.auto_updates {
            Task::perform(
                check_for_updates(Arc::clone(&self.paths), client.clone()),
                Message::CheckForUpdate,
            )
        } else {
            Task::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::CheckForUpdate(b) => {
                self.show_update_popup = b;
                Action::None
            },

            Message::UpdateNow => {
                Action::UpdateNeeded
            },

            Message::UpdateLater => {
                self.show_update_popup = false;
                Action::None
            },

            Message::LinkInputChanged(s) => {
                self.link_input = s;
                self.link_error = None;

                if self.link_input.is_empty() {
                    return Action::None;
                }

                if !self.paths.yt_dlp_exe.exists()
                || !self.paths.ffmpeg_dir.exists()
                || !self.paths.deno_exe.exists() {
                    self.show_update_popup = true;
                    error!("HOME: tools missing");
                    return Action::None;
                }

                match url::Url::parse(&self.link_input) {
                    Ok(_) => {
                        info!("HOME: querying link info");
                        self.loading_link_info = true;
                        Action::Run(
                            Task::perform(
                                yt_dlp::query_link_info(
                                    self.paths.yt_dlp_exe.clone(),
                                    self.paths.ffmpeg_dir.clone(),
                                    self.paths.deno_exe.clone(),
                                    self.link_input.clone(),
                                ),
                                Message::LinkInfoQueryFinished,
                            ),
                        )
                    },

                    Err(e) => {
                        error!("HOME: invalid url input -> {e}");
                        self.link_error = Some(LinkError::InvalidUrl);
                        Action::None
                    },
                }
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

                            LinkInfo::Playlist(i) => {
                                error!("playlist can not contain an other playlist");
                                self.playlist_link_info = None;
                            },

                            _ => self.playlist_link_info = Some(v),
                        }
                        
                    },

                    Err(e) => {
                        error!("failed to retrieve playlist link info -> {e}");
                        self.playlist_link_info = None;
                        self.link_error = Some(LinkError::InfoRetrievalFailed);
                    },
                }

                Action::None
            },

            Message::LinkInfoQueryFinished(r) => {
                self.loading_link_info = false;

                match r {
                    Ok(v) => {
                        info!("HOME: link info retrieved successfully");

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
                    },

                    Err(e) => {
                        error!("HOME: failed to retrieve link info -> {e}");
                        self.link_info = None;
                        self.link_error = Some(LinkError::InfoRetrievalFailed);
                    },
                }
                
                Action::None
            },

            Message::ThemeSelected(theme) => {
                self.settings.ui_theme = theme;
                Action::SettingsChanged(self.settings.clone())
            },

            Message::AutoUpdatesToggled(b) => {
                self.settings.auto_updates = b;
                Action::SettingsChanged(self.settings.clone())
            }

            Message::LanguageSelected(language) => {
                self.settings.ui_language = language;
                Action::SettingsChanged(self.settings.clone())
            },

            Message::ShowCreditsPopup(b) => {
                self.show_credits_popup = b;
                Action::None
            },

            Message::ShowUninstallPopup(b) => {
                self.show_uninstall_popup = b;
                Action::None
            }

            Message::Uninstall => {
                self.show_uninstall_popup = false;
                Action::Run(
                    Task::perform(
                        uninstall(),
                        Message::UninstallScriptLaunched,
                    ),
                )
            }

            Message::UninstallScriptLaunched(r) => {
                match r {
                    Ok(_) => {
                        info!("HOME: uninstall script successfully launched");
                        Action::Exit
                    },
                    Err(e) => {
                        error!("HOME: failed to launch update script -> {e}");
                        error_dialog("failed to uninstall");
                        Action::None
                    },
                }
            },

            Message::PasteLink => {
                info!("HOME: paste clipboard contents");
                Action::Run(
                    iced::clipboard::read()
                        .map(Message::ClipboardRead)
                )
            },

            Message::ClipboardRead(contents) => {
                match contents {
                    Some(v) => {
                        Action::Run(
                            Task::done(
                                Message::LinkInputChanged(v),
                            ),
                        )
                    },
                    None => Action::None,
                }
            },

            Message::VideoQualitySelected(q) => {
                self.selected_video_quality = q;
                Action::None
            },

            Message::AudioOnlyToggled(b) => {
                self.audio_only = b;
                Action::None
            },

            Message::VideoFormatSelected(f) => {
                self.selected_video_format = f;
                Action::None
            },

            Message::AudioFormatSelected(f) => {
                self.selected_audio_format = f;
                Action::None
            }

            Message::PlaylistItemSelected(i) => {
                info!("querying playlist link info");
                self.loading_playlist_link_info = true;

                self.selected_playlist_item = Some(Arc::clone(&i));
                Action::Run(
                    Task::perform(
                        yt_dlp::query_link_info(
                            self.paths.yt_dlp_exe.clone(),
                            self.paths.ffmpeg_dir.clone(),
                            self.paths.deno_exe.clone(),
                            i.url.clone(),
                        ),
                        Message::PlaylistLinkInfoQueryFinished,
                    ),
                )
            },

            Message::Debug => Action::None,
        }
    }

    // TODO: Translate
    pub fn view(&self, translation: &Translation, images: &Images) -> Element<'_, Message> {
        let info_panel: Element<'_, Message> = if let Some(e) = &self.link_error {
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
        } else if self.loading_link_info {
            column![
                Circular::new(),
                space().height(30),
                text("Loading link..."),
                space().height(Length::Fill),
            ]
            .align_x(Horizontal::Center)
            .into()
        } else if let Some(li) = &self.link_info {
            match li {
                LinkInfo::Playlist(playlist) => self.playlist_panel(translation, playlist),
                LinkInfo::Video(video) => self.video_info_panel(translation, video, true),
                LinkInfo::Audio(audio) => self.audio_info_panel(translation, audio, true),
            }
        } else {
            space().into()
        };
        
        let base = column![
            self.menu_bar(translation),
            space().height(20),
            center(
                scrollable(column![
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                    "THIS",
                ])
                .width(400)
                .height(200),
            )
            .height(Length::FillPortion(1)),

            center(
                row![
                    space().width(Length::Fill),
                    row![
                        space().width(50),
                        text_input(translation.home_screen_link_input_placeholder, &self.link_input)
                            .on_input(Message::LinkInputChanged),
                        space().width(5),
                        button(
                            center(
                                image(
                                    images.paste.clone(),
                                ),
                            ),
                        )
                        .width(50)
                        .height(30)
                        .on_press(Message::PasteLink),    
                    ]
                    .width(Length::FillPortion(3)),
                    space().width(Length::Fill),
                ]
                .align_y(Vertical::Center),
            )
            .height(Length::Shrink),
            
            center(info_panel)
                .height(Length::FillPortion(2)),
        ];

        let base = if self.show_credits_popup {
            self.modal(base, self.credits_popup(translation))
        } else {
            base.into()
        };

        let base = if self.show_uninstall_popup {
            self.modal(base, self.uninstall_popup(translation))
        } else {
            base.into()
        };

        if self.show_update_popup {
            let popup = self.update_popup(translation);
            self.modal(base, popup)
        } else {
            base.into()
        }
    }

    fn playlist_panel<'a>(
        &'a self,
        translation: &Translation,
        info: &'a PlaylistInfo,
    ) -> Element<'a, Message> {
        let selection: Element<'_, Message> = combo_box(
            &self.playlist_items,
            "Select playlist item...",
            self.selected_playlist_item.as_ref(),
            Message::PlaylistItemSelected,
        )
        .menu_height(200)
        .into();

        let info_panel: Element<'_, Message> = if let Some(e) = &self.link_error {
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
        } else if self.loading_playlist_link_info {
            column![
                Circular::new(),
                space().height(30),
                text("Loading link..."),
                space().height(Length::Fill),
            ]
            .align_x(Horizontal::Center)
            .into()
        } else if let Some(li) = &self.playlist_link_info {
            match li {
                LinkInfo::Playlist(_) => space().into(),
                LinkInfo::Video(video) => self.video_info_panel(translation, video, false),
                LinkInfo::Audio(audio) => self.audio_info_panel(translation, audio, false),
            }
        } else {
            space().height(Length::Fill).into()
        };
        
        row![
            space().width(Length::FillPortion(1)),
            column![
                text(format!("{}", info.name)),
                row![
                    space().width(50),
                    selection,
                    space().width(50),
                ]
                .align_y(Vertical::Center),

                space().height(30),
                info_panel,
            ]
            .align_x(Horizontal::Center)
            .width(Length::FillPortion(2)),
            space().width(Length::FillPortion(1)),
        ]
        .into()
    }

    fn audio_info_panel<'a>(
        &'a self,
        translation: &Translation,
        info: &'a AudioInfo,
        squish: bool,
    ) -> Element<'a, Message> {
        let title = match &info.title {
            Some(v) => v,
            None => "Unknown",
        };

        let channel = match &info.channel {
            Some(v) => v,
            None => "Unknown",
        };

        let options: Element<'_, Message> =
            row![
                text("Format"),
                space().width(5),
                pick_list(
                    AudioFileType::file_types(),
                    Some(&self.selected_audio_format),
                    Message::AudioFormatSelected,
                ),
            ]
            .align_y(Vertical::Center)
            .into();

        row![
            if squish { space().width(Length::FillPortion(1)) } else { space() },
            column![
                text(format!("{}", title)).size(20),
                text(format!("{} {}", "by", channel)),
                space().height(30),
                options,
                space().height(30),
                button("Download").on_press(Message::Debug),
                space().height(Length::Fill),
            ]
            .align_x(Horizontal::Center)
            .width(Length::FillPortion(2)),
            if squish { space().width(Length::FillPortion(1)) } else { space() },
        ]
        .into()
    }

    fn video_info_panel<'a>(
        &'a self,
        translation: &Translation,
        info: &'a VideoInfo,
        squish: bool,
    ) -> Element<'a, Message> {
        let title = match &info.title {
            Some(v) => v,
            None => "Unknown",
        };

        let channel = match &info.channel {
            Some(v) => v,
            None => "Unknown",
        };

        let options: Element<'_, Message> = if !self.audio_only {
            row![
                text("Quality"),
                space().width(5),
                pick_list(
                    info.qualities(),
                    Some(&self.selected_video_quality),
                    Message::VideoQualitySelected,
                )
                .menu_height(150),

                space().width(50),

                text("Format"),
                space().width(5),
                pick_list(
                    VideoFileType::file_types(),
                    Some(&self.selected_video_format),
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
                    Some(&self.selected_audio_format),
                    Message::AudioFormatSelected,
                ),
            ]
            .align_y(Vertical::Center)
            .into()
        };

        row![
            if squish { space().width(Length::FillPortion(1)) } else { space() },
            column![
                text(format!("{}", title)).size(20),
                text(format!("{} {}", "by", channel)),

                space().height(30),

                checkbox(self.audio_only)
                    .label("Audio only")
                    .on_toggle(Message::AudioOnlyToggled),
            
                space().height(30),
                options,
                space().height(30),
                button("Download").on_press(Message::Debug),
                space().height(Length::Fill),
            ]
            .align_x(Horizontal::Center)
            .width(Length::FillPortion(2)),
            if squish { space().width(Length::FillPortion(1)) } else { space() },
        ]
        .into()
    }

    fn uninstall_popup(&self, translation: &Translation) -> Element<'_, Message> {
        mouse_area(
            center(
                center(
                    column![
                        space().height(5),
                        text(translation.home_screen_about_uninstall)
                            .size(30),
                        space().height(Length::Fill),
                        text(translation.home_screen_uninstall_caption),
                        space().height(Length::Fill),
                        row![
                            space().width(Length::FillPortion(2)),
                            button(translation.general_yes)
                                .on_press(Message::Uninstall),
                            space().width(Length::FillPortion(1)),
                            button(translation.general_no)
                                .on_press(Message::ShowUninstallPopup(false)),
                            space().width(Length::FillPortion(2)),
                        ]
                        .align_y(Vertical::Center),
                        space().height(5)
                    ]
                    .align_x(Horizontal::Center)
                )
                .style(|theme: &iced::Theme| {
                    let pal = theme.extended_palette();
                    container::Style {
                        background: Some(pal.background.base.color.into()),
                        border: iced::border::rounded(10),
                        ..Default::default()
                    }
                })
                .width(350)
                .height(200)
            )
            .style(container::transparent)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .on_press(Message::ShowUninstallPopup(false))
        .into()
    }
    
    fn credits_popup(&self, translation: &Translation) -> Element<'_, Message> {
        mouse_area(
            center(
                center(
                    column![
                        space().height(5),
                        text(translation.home_screen_about_credits)
                            .size(30),
                        space().height(Length::Fill),
                        text(translation.home_screen_credits_content),
                        text("Fathema Khanom - Flaticon"),
                        text("paonkz - Flaticon"),
                        text("Roundicons - Flaticon"),
                        space().height(Length::Fill),
                        button(translation.general_close)
                            .on_press(Message::ShowCreditsPopup(false)),
                        space().height(5),
                    ]
                    .align_x(Horizontal::Center)
                )
                .style(|theme: &iced::Theme| {
                    let pal = theme.extended_palette();
                    container::Style {
                        background: Some(pal.background.base.color.into()),
                        border: iced::border::rounded(10),
                        ..Default::default()
                    }
                })
                .width(400)
                .height(200),
            )
            .style(container::transparent)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .on_press(Message::ShowCreditsPopup(false))
        .into()
    }

    fn menu_bar(&self, translation: &Translation) -> Element<'_, Message> {
        let settings_menu = Item::with_menu(
            self.menu_button(translation.home_screen_menu_settings, Message::Debug),
            Menu::new(vec![
                Item::new(
                    row![
                        space().width(5),
                        text(translation.home_screen_settings_color_scheme),
                        space().width(Length::Fill),
                        pick_list(
                            [crate::Theme::Dark, crate::Theme::Light, crate::Theme::Auto],
                            Some(&self.settings.ui_theme),
                            Message::ThemeSelected,
                        ),
                        space().width(5),
                    ]
                    .align_y(Vertical::Center),
                ),

                Item::new(
                    row![
                        space().width(5),
                        text(translation.home_screen_settings_language),
                        space().width(30),
                        pick_list(
                            [crate::lang::Language::English, crate::lang::Language::German],
                            Some(&self.settings.ui_language),
                            Message::LanguageSelected,
                        ),
                        space().width(5),
                    ]
                    .align_y(Vertical::Center),
                ),

                Item::new(
                    row![
                        space().width(5),
                        text(translation.home_screen_settings_auto_updates),
                        space().width(Length::Fill),
                        toggler(self.settings.auto_updates)
                            .on_toggle(Message::AutoUpdatesToggled),
                        space().width(5),
                    ]
                    .align_y(Vertical::Center),
                ),
            ])
            .width(Length::Shrink)
            .spacing(10.0)
            .offset(5.0)
            .padding(Padding::new(0.0).bottom(10)),
        );

        let about_menu = Item::with_menu(
            self.menu_button(translation.home_screen_menu_about, Message::Debug),
            Menu::new(vec![
                Item::new(row![
                    self.menu_button(
                        translation.home_screen_about_credits,
                        Message::ShowCreditsPopup(true),
                    ),
                    space().width(Length::Fill),
                ].align_y(Vertical::Center)),

                Item::new(
                    self.menu_button(
                        translation.home_screen_about_uninstall,
                        Message::ShowUninstallPopup(true),
                    ),
                ),
            ])
            .width(Length::Shrink)
            .spacing(5.0)
            .offset(5.0),
        );

        let bar = MenuBar::new(vec![settings_menu, about_menu]).spacing(2);

        bar.into()
    }

    fn menu_button<'a>(&self, content: impl Into<Element<'a, Message>>, message: Message) -> Element<'a, Message> {
        button(content)
            .padding([4, 8])
            .style(|theme, status| {
                let pal = theme.extended_palette();
                let base = button::Style {
                    text_color: pal.background.base.text,
                    border: iced::Border::default().rounded(6.0),
                    ..Default::default()
                };

                match status {
                    button::Status::Active => base.with_background(Color::TRANSPARENT),
                    button::Status::Hovered => base.with_background(Color::from_rgb(
                        (pal.primary.weak.color.r * 1.2).clamp(0.0, 1.0),
                        (pal.primary.weak.color.g * 1.2).clamp(0.0, 1.0),
                        (pal.primary.weak.color.b * 1.2).clamp(0.0, 1.0),
                    )),
                    button::Status::Disabled => base.with_background(Color::from_rgb(0.5, 0.5, 0.5)),
                    button::Status::Pressed => base.with_background(pal.primary.weak.color),
                }
            })
            .on_press(message)
            .into()
    }

    fn update_popup(&self, translation: &Translation) -> Element<'_, Message> {
        center(
            column![
                space().height(20),
                text(translation.home_screen_update_popup_caption).size(30),
                space().height(Length::Fill),
                row![
                    space().width(Length::Fill),
                    button(translation.home_screen_update_popup_button_update_now).on_press(Message::UpdateNow),
                    space().width(Length::Fill),
                    button(translation.home_screen_update_pupup_button_update_later).on_press(Message::UpdateLater),
                    space().width(Length::Fill),
                ],
                space().height(20),
            ]
            .align_x(Horizontal::Center)
        )
        .style(|theme: &iced::Theme| {
            let pal = theme.extended_palette();
            container::Style {
                background: Some(pal.background.base.color.into()),
                border: iced::border::rounded(10),
                ..Default::default()
            }
        })
        .width(300)
        .height(200)
        .into()
    }

    fn modal<'a>(
        &self,
        base: impl Into<Element<'a, Message>>,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        stack![
            base.into(),
            opaque(
                center(opaque(content))
                    .style(|_| {
                        container::Style {
                            background: Some(
                                Color {
                                    a: 0.5,
                                    ..Color::BLACK
                                }
                                .into(),
                            ),
                            ..Default::default()
                        }
                    })
            ),
        ].into()
    }
}

async fn check_for_updates(paths: Arc<Paths>, client: Client) -> bool {
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

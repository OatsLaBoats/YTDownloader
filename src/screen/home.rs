use std::sync::Arc;

use iced::{Color, Length, Padding};
use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Task};
use iced::widget::*;
use iced::widget::column;
use iced_aw::menu::Item;
use iced_aw::*;
use reqwest::Client;
use tracing::{info, error};

use crate::command::{deno, yt_dlp};
use crate::{Paths, Settings, github};
use crate::lang::Translation;

pub struct Screen {
    paths: Arc<Paths>,
    settings: Settings,
    show_update_popup: bool,
    show_credits_popup: bool,
    link_input: String,
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
    ShowCredits(bool),
    Debug,
}

pub enum Action {
    None,
    UpdateNeeded,
    SettingsChanged(Settings),
}

impl Screen {
    pub fn new(paths: Arc<Paths>, settings: Settings) -> Self {
        Self {
            paths,
            settings,
            show_update_popup: false,
            show_credits_popup: false,
            link_input: String::new(),
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

            Message::ShowCredits(b) => {
                self.show_credits_popup = b;
                Action::None
            },

            Message::Debug => Action::None,
        }
    }

    pub fn view(&self, translation: &Translation) -> Element<'_, Message> {
        let base = column![
            self.menu_bar(translation),
            center(
                text_input(translation.home_screen_link_input_placeholder, &self.link_input)
                    .on_input(Message::LinkInputChanged)
            ).padding(30),
        ];

        let base = if self.show_credits_popup {
            self.modal(base, self.credits_popup(translation))
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
                        space().height(Length::Fill),
                        button(translation.home_screen_credits_close)
                            .on_press(Message::ShowCredits(false)),
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
                .height(200)
            )
            .style(container::transparent)
            .width(Length::Fill)
            .height(Length::Fill)
        )
            .on_press(Message::ShowCredits(false))
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
                    ].align_y(Vertical::Center),
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
                    ].align_y(Vertical::Center),
                ),

                Item::new(
                    row![
                        space().width(5),
                        text(translation.home_screen_settings_auto_updates),
                        space().width(Length::Fill),
                        toggler(self.settings.auto_updates)
                            .on_toggle(Message::AutoUpdatesToggled),
                        space().width(5),
                    ].align_y(Vertical::Center),
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
                Item::new(self.menu_button(translation.home_screen_about_credits, Message::ShowCredits(true))),
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
                text(translation.home_screen_popup_caption).size(30),
                space().height(Length::Fill),
                row![
                    space().width(Length::Fill),
                    button(translation.home_screen_pupup_button_update_now).on_press(Message::UpdateNow),
                    space().width(Length::Fill),
                    button(translation.home_screen_pupup_button_update_later).on_press(Message::UpdateLater),
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

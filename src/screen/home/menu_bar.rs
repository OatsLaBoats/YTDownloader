use iced::alignment::Vertical;
use iced::{Color, Element, Length, Padding};
use iced::widget::*;
use iced_aw::*;
use iced_aw::menu::Item;

use crate::Settings;
use crate::lang::Translation;
use super::popup::PopupId;

pub struct State {
    settings: Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateSettings(Settings),
    
    ThemeSelected(crate::Theme),
    LanguageSelected(crate::Language),
    AutoUpdatesToggled(bool),
    ShowPopup(PopupId, bool),
    Debug,
}

pub enum Action {
    None,
    SettingsChanged(Settings),
    ShowPopup(PopupId, bool),
}

impl State {
    pub fn new(
        settings: Settings,
    ) -> Self {
        Self {
            settings,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::UpdateSettings(settings) => {
                self.settings = settings;
                Action::None
            },
            
            Message::ThemeSelected(theme) => {
                self.settings.ui_theme = theme;
                Action::SettingsChanged(self.settings.clone())
            },

            Message::LanguageSelected(language) => {
                self.settings.ui_language = language;
                Action::SettingsChanged(self.settings.clone())
            },

            Message::AutoUpdatesToggled(b) => {
                self.settings.auto_updates = b;
                Action::SettingsChanged(self.settings.clone())
            },

            Message::ShowPopup(p, b) => {
                Action::ShowPopup(p, b)
            },
            
            Message::Debug => Action::None,
        }
    }

    pub fn view(&self, translation: &Translation) -> Element<'_, Message> {
        let settings_menu = Item::with_menu(
            menu_button(translation.home_screen_menu_settings, Message::Debug),
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
            menu_button(translation.home_screen_menu_about, Message::Debug),
            Menu::new(vec![
                Item::new(row![
                    menu_button(
                        translation.home_screen_about_credits,
                        Message::ShowPopup(super::POPUP_CREDITS, true),
                    ),
                ].align_y(Vertical::Center)),

                Item::new(
                    menu_button(
                        translation.home_screen_about_uninstall,
                        Message::ShowPopup(super::POPUP_UNINSTALL, true),
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
}

fn menu_button<'a>(content: impl Into<Element<'a, Message>>, message: Message) -> Element<'a, Message> {
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

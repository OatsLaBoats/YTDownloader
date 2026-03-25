use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Task};
use iced::widget::*;
use iced::widget::column;
use reqwest::Client;
use tracing::{info, error};

use crate::command::yt_dlp;
use crate::platform::windows::{error_dialog, uninstall};
use crate::{Images, Paths, Settings};
use crate::lang::Translation;
use super::modal;

mod download;
mod tasks;
mod popup;
mod menu_bar;
mod info_panel;

use download::{DownloadInfo};
use tasks::check_for_updates;

// TODO: Pause, and cancel download buttons
// You can query progress using _percent, eta, tmpfilename
// Query video info using templaes instead of json because it seem too error prone
//
// TODO: Hide download list when nothing is downloading
// TODO: Make link info cancelable
//
// TODO: Refactor

const POPUP_UPDATE: usize = 0;
const POPUP_CREDITS: usize = 1;
const POPUP_UNINSTALL: usize = 2;

pub struct Screen {
    paths: Arc<Paths>,
    settings: Settings,

    popups: [popup::State; 3],

    menu_bar: menu_bar::State,
    
    link_input: String,

    info_panel: info_panel::State,
}

#[derive(Clone, Debug)]
pub enum Message {
    CheckForUpdateFinished(bool),
    
    PopupMessage(popup::Message),
    ShowPopup(popup::PopupId, bool),

    MenuMessage(menu_bar::Message),

    UninstallScriptLaunched(crate::platform::windows::Result<()>),

    LinkInputChanged(String),
    PasteLink,
    ClipboardRead(Option<String>),

    InfoPanelMessage(info_panel::Message),

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
            paths: Arc::clone(&paths),
            settings: settings.clone(),

            popups: [
                popup::State::new(POPUP_UPDATE),
                popup::State::new(POPUP_CREDITS).close_on_click(),
                popup::State::new(POPUP_UNINSTALL),
            ],

            menu_bar: menu_bar::State::new(settings.clone()),
            
            link_input: String::new(),

            info_panel: info_panel::State::new(settings, paths),
        }
    }

    pub fn start(&mut self, client: &Client) -> Task<Message> {
        if self.settings.auto_updates {
            Task::perform(
                check_for_updates(Arc::clone(&self.paths), client.clone()),
                Message::CheckForUpdateFinished,
            )
        } else {
            Task::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::InfoPanelMessage(message) => {
                let action = self.info_panel.update(message);
                match action {
                    info_panel::Action::None => Action::None,
                    info_panel::Action::Run(task) => Action::Run(task.map(Message::InfoPanelMessage)),
                    info_panel::Action::SettingsChanged(settings) => Action::SettingsChanged(settings),
                    info_panel::Action::Download(info) => {
                        info!("{info:?}");
                        Action::None
                    },
                }
            },
            
            Message::MenuMessage(message) => {
                let action = self.menu_bar.update(message);
                match action {
                    menu_bar::Action::None => Action::None,
                    menu_bar::Action::SettingsChanged(settings) => Action::SettingsChanged(settings),
                    menu_bar::Action::ShowPopup(p, b) => Action::Run(
                        Task::done(Message::ShowPopup(p, b))
                    ),
                }
            },
            
            Message::ShowPopup(p, b) => {
                self.popups[p].set_visibility(b);
                Action::None
            },
            
            Message::PopupMessage(m) => {
                let (id, kind) = m.0;
                let p = &mut self.popups[id];
                let action = p.update(kind);

                match action {
                    popup::Action::None => Action::None,
                    popup::Action::Pressed(btn) => {
                        match id {
                            POPUP_UPDATE => {
                                p.set_visibility(false);
                                if btn == 0 {
                                    Action::UpdateNeeded
                                } else {
                                    Action::None
                                }
                            },

                            POPUP_CREDITS => {
                                p.set_visibility(false);
                                Action::None
                            },

                            POPUP_UNINSTALL => {
                                p.set_visibility(false);
                                if btn == 0 {
                                    Action::Run(
                                        Task::perform(
                                            uninstall(),
                                            Message::UninstallScriptLaunched,
                                        ),
                                    )
                                } else {
                                    Action::None
                                }
                            },
                            
                            _ => Action::None,
                        }
                    },
                }
            },
            
            Message::CheckForUpdateFinished(b) => {
                Action::Run(
                    Task::done(
                        Message::ShowPopup(POPUP_UPDATE, b),
                    ),
                )
            },

            Message::LinkInputChanged(s) => {
                self.link_input = s;

                if self.link_input.is_empty() {
                    return Action::None;
                }

                if !self.paths.yt_dlp_exe.exists()
                || !self.paths.ffmpeg_dir.exists()
                || !self.paths.deno_exe.exists() {
                    error!("HOME: tools missing");
                    return Action::Run(
                        Task::done(
                            Message::ShowPopup(POPUP_UPDATE, true),
                        ),
                    );
                }

                match self.info_panel.start(self.link_input.clone()) {
                    Some(task) => Action::Run(task.map(Message::InfoPanelMessage)),
                    None => Action::None,
                }
            },

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

            Message::Debug => Action::None,
        }
    }

    // TODO: Translate
    pub fn view<'a>(&'a self, translation: &'a Translation, images: &'a Images) -> Element<'a, Message> {
        let panel = self.info_panel.view(translation).map(Message::InfoPanelMessage);

        let menu = self.menu_bar.view(translation).map(Message::MenuMessage);

        let link_input =
            row![
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
            .align_y(Vertical::Center);
       
        let base = column![
            menu,
            link_input,
            panel,
        ]
        .spacing(30);

        // Popups
        let base = if self.popups[POPUP_CREDITS].is_visible() {
            let popup = 
                self.popups[POPUP_CREDITS].view(
                    translation.home_screen_about_credits,
                    column![
                        text(translation.home_screen_credits_content),
                        text("Fathema Khanom - Flaticon"),
                        text("paonkz - Flaticon"),
                        text("Roundicons - Flaticon"),
                    ]
                    .align_x(Horizontal::Center),
                    vec![translation.general_close],
                    400,
                    200,
               );

            modal(base, popup.map(Message::PopupMessage))
        } else {
            base.into()
        };

        let base = if self.popups[POPUP_UNINSTALL].is_visible() {
            let popup = 
                self.popups[POPUP_UNINSTALL].view(
                    translation.home_screen_about_uninstall,
                    text(translation.home_screen_uninstall_caption),
                    vec![translation.general_yes, translation.general_no],
                    350,
                    200,
               );

            modal(base, popup.map(Message::PopupMessage))
        } else {
            base.into()
        };

        if self.popups[POPUP_UPDATE].is_visible() {
            let popup = 
                self.popups[POPUP_UPDATE].view(
                    translation.home_screen_update_popup_title,
                    space(),
                    vec![
                        translation.home_screen_update_popup_button_update_now,
                        translation.home_screen_update_pupup_button_update_later,
                    ],
                    300,
                    150,
               );

            modal(base, popup.map(Message::PopupMessage))
        } else {
            base.into()
        }
    }
}

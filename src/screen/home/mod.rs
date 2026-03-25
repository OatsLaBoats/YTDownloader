use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::{Color, Element, Length, Task};
use iced::widget::*;
use iced::widget::column;
use iced_aw::*;
use reqwest::Client;
use tracing::{info, error};

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

struct IdGenerator {
    cache: Vec<usize>,
    latest: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            latest: 0,
        }
    }

    pub fn id(&mut self) -> usize {
        if let Some(id) = self.cache.pop() {
            return id;
        }
        
        self.latest += 1;
        self.latest
    }

    pub fn free(&mut self, id: usize) {
        self.cache.push(id);
    }
}

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

    show_side_bar: bool,
    ids: IdGenerator,
}

#[derive(Clone, Debug)]
pub enum Message {
    UpdateSettings(Settings),
    
    CheckForUpdateFinished(bool),
    
    PopupMessage(popup::Message),
    ShowPopup(popup::PopupId, bool),

    MenuMessage(menu_bar::Message),

    UninstallScriptLaunched(crate::platform::windows::Result<()>),

    LinkInputChanged(String),
    PasteLink,
    ClipboardRead(Option<String>),

    InfoPanelMessage(info_panel::Message),

    ShowSideBar(bool),

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

            show_side_bar: false,
            ids: IdGenerator::new(),
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
            Message::ShowSideBar(b) => {
                self.show_side_bar = b;
                Action::None
            },
            
            // Sync settings with children
            Message::UpdateSettings(settings) => {
                let task1 = Task::done(info_panel::Message::UpdateSettings(settings.clone())).map(Message::InfoPanelMessage);
                let task2 = Task::done(menu_bar::Message::UpdateSettings(settings)).map(Message::MenuMessage);

                Action::Run(
                    task1.chain(task2)
                )
            },
            
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
        let panel = self.info_panel.view(translation, images).map(Message::InfoPanelMessage);

        let menu = self.menu_bar.view(translation).map(Message::MenuMessage);

        let link_input =
            row![
                text_input(translation.home_screen_link_input_placeholder, &self.link_input)
                    .on_input(Message::LinkInputChanged),
                space().width(5),
                button(
                    center(image(images.paste.clone()))
                        .width(30)
                        .height(21),
                )
                .on_press(Message::PasteLink),    
            ]
            .align_y(Vertical::Center);

        let link_input = ContextMenu::new(link_input, || {
            button(translation.context_menu_paste)
                .on_press(Message::PasteLink)
                .into()
        });

        let layout = row![
            space().width(Length::FillPortion(3)),

            column![
                space().height(Length::Fill),
                link_input,
                space().height(50),
                panel,
                space().height(Length::Fill),
            ]
            .width(Length::FillPortion(7))
            .align_x(Horizontal::Center),

            space().width(Length::FillPortion(3)),
        ];
       
        let base = column![
            menu,
            layout,
        ];

        let side_bar = if self.show_side_bar {
            opaque(
                container(
                    column![
                        center(
                            text(translation.info_panel_side_bar_title)
                                .size(25),
                        )
                        .width(Length::Fill)
                        .height(Length::Shrink),

                        space().height(10),
                            
                        scrollable(
                            column![
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                                "HELLO",
                            ]
                            .width(Length::Fill)
                            .align_x(Horizontal::Center),
                        ),
                    ],
                )
                .style(|_| {
                    container::Style {
                        background: Some(
                            Color {
                                a: 0.2,
                                ..Color::BLACK
                            }
                            .into(),
                        ),
                        ..Default::default()
                    }
                })
                .height(Length::Fill)
                .width(Length::FillPortion(2)),
            )
        } else {
            space().into()
        };

        // Download side bar
        let side_bar = stack![
            base,

            row![
                space().width(Length::FillPortion(7)),

                button(
                    if self.show_side_bar {
                        center(image(images.arrow_right.clone()))
                    } else {
                        center(image(images.arrow_left.clone()))
                    }
                )
                .width(50)
                .height(50)
                .on_press(Message::ShowSideBar(!self.show_side_bar))
                .style(|theme: &iced::Theme, _| {
                    let pal = theme.extended_palette();
                    button::Style {
                        text_color: pal.background.base.text,
                        background: Some(
                            Color {
                                a: 0.2,
                                ..Color::BLACK
                            }
                            .into(),
                        ),
                        ..Default::default()
                    }
                }),

                side_bar,
            ],
        ];

        self.popups(translation, side_bar)
    }

    fn popups<'a>(&'a self, translation: &'a Translation, base: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        let base = if self.popups[POPUP_CREDITS].is_visible() {
            let popup = 
                self.popups[POPUP_CREDITS].view(
                    translation.home_screen_about_credits,
                    column![
                        text(translation.home_screen_credits_content),
                        text("Fathema Khanom - Flaticon"),
                        text("paonkz - Flaticon"),
                        text("Roundicons - Flaticon"),
                        text("Freepik - Flaticon"),
                        text("ariefstudio - Flaticon"),
                        text("joalfa - Flaticon"),
                    ]
                    .align_x(Horizontal::Center),
                    vec![translation.general_close],
                    400,
                    300,
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

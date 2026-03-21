use std::sync::Arc;

use iced::{Element, Task};
use iced::widget::*;
use reqwest::Client;

use crate::{Paths, Settings};
use crate::lang::Translation;

pub struct Screen {
    paths: Arc<Paths>,
    settings: Settings,
}

#[derive(Clone, Debug)]
pub enum Message {
    CheckForUpdate(bool),
}

pub enum Action {
    None,
}

impl Screen {
    pub fn new(paths: Arc<Paths>, settings: Settings) -> Self {
        Self {
            paths,
            settings,
        }
    }

    pub fn start(&mut self, client: Client) -> Task<Message> {
        if self.settings.auto_updates {
            Task::perform(
                check_for_updates(Arc::clone(&self.paths), client),
                Message::CheckForUpdate,
            )
        } else {
            Task::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        Action::None
    }

    pub fn view(&self, translation: &Translation) -> Element<'_, Message> {
        container("Hello").into()
    }
}

async fn check_for_updates(paths: Arc<Paths>, client: Client) -> bool {
    false
}

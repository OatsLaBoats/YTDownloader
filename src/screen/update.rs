use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::*;
use iced::widget::*;
use iced::widget::column;
use iced::widget::space::*;

use crate::AppPaths;
use crate::lang::{TextDatabase, Translation};

pub struct Screen {
    first_install: bool,
    paths: Arc<AppPaths>,
    progess: f32,
}

impl Screen {
    // Unlike App screens can have state passed into them so we don't use the default trait
    pub fn new(
        first_install: bool,
        paths: Arc<AppPaths>,
    ) -> Self {
        Self {
            first_install,
            paths,
            progess: 0.0,
        }
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        todo!()
    }

    
    pub fn view(&self, translation: &Translation) -> Element<'_, Message> {
        center(
            column![
                text(if self.first_install {
                    translation.update_screen_install_label
                } else {
                    translation.update_screen_update_label
                })
                .size(30),

                space().height(30),
            
                progress_bar(0.0f32..=1.0f32, self.progess),
            ]
            .align_x(Horizontal::Center)
            .padding(30)
        )
        .into()
    }
}

#[derive(Clone)]
pub enum Message {
    
}

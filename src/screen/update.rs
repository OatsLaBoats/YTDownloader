use iced::*;
use iced::widget::*;

pub struct Screen {
    
}

impl Screen {
    // Unlike App screens can have state passed into them so we don't use the default trait
    pub fn new() -> Self {
        Self {
            
        }
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        todo!()
    }

    
    pub fn view(&self) -> Element<'_, Message> {
        todo!()
    }
}

pub enum Message {
    
}

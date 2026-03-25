use iced::alignment::{Horizontal, Vertical};
use iced::{Element, Length};
use iced::widget::*;
use iced::widget::column;

pub struct State {
    id: PopupId,
    show: bool,
    close_on_click: bool,
}

pub type PopupId = usize;
pub type ButtonIndex = usize;

pub enum Action {
    None,
    Pressed(ButtonIndex),
}

#[derive(Clone, Debug)]
pub struct Message(pub (PopupId, MessageKind));

impl Message {
    pub fn map<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

#[derive(Clone, Debug)]
pub enum MessageKind {
    Show(bool),
    Pressed(ButtonIndex),
}

impl State {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            show: false,
            close_on_click: false,
        }
    }

    #[allow(unused)]
    pub fn show(mut self) -> Self {
        self.show = true;
        self
    }

    pub fn close_on_click(mut self) -> Self {
        self.close_on_click = true;
        self
    }

    pub fn set_visibility(&mut self, show: bool) {
        self.show = show;
    }

    pub fn show_message(&self, show: bool) -> Message {
        Message((
            self.id,
            MessageKind::Show(show),
        ))
    }

    pub fn is_visible(&self) -> bool {
        self.show
    }

    pub fn update(&mut self, message: MessageKind) -> Action {
        match message {
            MessageKind::Show(b) => {
                self.show = b;
                Action::None
            },

            MessageKind::Pressed(index) => {
                Action::Pressed(index)
            },
        }
    }

    pub fn view<'a>(
        &'a self,
        title: &'a str,
        caption: impl Into<Element<'a, Message>>,
        button_names: Vec<&'a str>,
        width: impl Into<Length>,
        height: impl Into<Length>,
    ) -> Element<'a, Message> {
        let mut  buttons = Row::new();
        buttons = buttons.push(space().width(Length::FillPortion(2)));

        let len = button_names.len();
        for (index, name) in button_names.iter().enumerate() {
            buttons = buttons.push(
                button(
                    text(*name),
                )
                .on_press(Message((self.id, MessageKind::Pressed(index)))),
            );

            if index < len - 1 {
                buttons = buttons.push(space().width(Length::FillPortion(1)));
            }
        }
        
        buttons = buttons.push(space().width(Length::FillPortion(2)));
        
        let popup = center(
            column![
                space().height(5),
                text(title)
                    .size(30),
                space().height(Length::Fill),
                caption.into(),
                space().height(Length::Fill),
                buttons
                    .align_y(Vertical::Center),
                space().height(10),
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
        .width(width)
        .height(height);

        let result: Element<'_, Message> = if self.close_on_click {
            mouse_area(
            center(popup)
                .style(container::transparent)
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .on_press(Message((self.id, MessageKind::Show(false))))
            .into()
        } else {
            center(popup)
                .style(container::transparent)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        result
    }
}

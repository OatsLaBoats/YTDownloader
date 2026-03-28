pub mod home;
pub mod update;

use std::time::Duration;

use iced::{Color, Element, Length};
use iced::widget::*;

pub enum Screen {
    Update(update::Screen),
    Home(home::Screen),
}

impl Default for Screen {
    fn default() -> Self {
        Self::Update(update::Screen::default())
    }
}

const TOOLTIP_DELAY: Duration = Duration::from_millis(400);

// Common widgets
fn modal<'a, Message: 'a>(
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
    ]
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

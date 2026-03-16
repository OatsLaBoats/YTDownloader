use iced::*;
use iced::widget::*;
use iced::widget::column;

fn main() -> iced::Result {
    let icon_data = include_bytes!("../res/YTDownloader.png");
    let icon = iced::window::icon::from_file_data(icon_data, None).ok();
    
    iced::application(App::default, App::update, App::view)
        .window(iced::window::Settings {
            icon: icon,
            ..Default::default()
        })
        .title("YT Downloader")
        .theme(|_: &App| Theme::Dark)
        .run()
}

struct App {
    link: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            link: String::new(),
        }
    }
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LinkBoxChanged(s) => self.link = s,
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        center(
            column![
                text_input("Link", &self.link)
                    .on_input(Message::LinkBoxChanged),
            ]
            .padding(20),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}

#[derive(Clone)]
enum Message {
    LinkBoxChanged(String),
}

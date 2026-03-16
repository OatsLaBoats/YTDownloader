use iced::*;
use iced::widget::*;
use iced::widget::column;
use yt_downloader::platform::windows;

fn main() -> iced::Result {
    let icon_data = include_bytes!("../res/YTDownloader.png");
    let icon = iced::window::icon::from_file_data(icon_data, None).ok();
    
    iced::application(App::new, App::update, App::view)
        .window(iced::window::Settings {
            icon: icon,
            ..Default::default()
        })
        .theme(|_: &App| Theme::Dark)
        .run()
}

struct App {
    link: String,
}

impl App {
    fn new() -> Self {
        Self {
            link: String::new(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LinkBoxChanged(s) => self.link = s,
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        center(
            column![
                text("Paste link:"),
                text_input("", &self.link)
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

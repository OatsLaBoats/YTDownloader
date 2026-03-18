#![windows_subsystem = "console"]

use std::path::PathBuf;

use iced::*;
use iced::widget::*;
use iced::widget::column;

use yt_downloader::lang::*;
use yt_downloader::screen::*;
use yt_downloader::platform::windows::*;

use tracing::{error, info};

// TODO: There seems to be a memory leak somewhere when resizing the window

fn main() -> iced::Result {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subsriber failed");

    let icon_data = include_bytes!("../res/YTDownloader.png");
    let icon = iced::window::icon::from_file_data(icon_data, None).ok();
    
    iced::application(App::default, App::update, App::view)
        .window(iced::window::Settings {
            icon: icon,
            ..Default::default()
        })
        .title("YT Downloader")
        .run()
}

// Holds the state for the whole app.
struct App {
    languages: TextDatabase,
    active_language: Language,

    downloads_path: PathBuf,
    appdata_path: PathBuf,
    downloader_path: PathBuf,
    
    active_screen: Screen,
}

impl Default for App {
    // NOTE: During initialization all errors are fatal. We need to make sure they are kept to a minimum.
    fn default() -> Self {
        // Ensure we can run the install script. In the future maybe support other ways
        // like bash or cmd
        if let Err(e) = which::which("powershell") {
            error!("Failed to find powershell executable {}", e);
            error_dialog("Failed to find powershell executable");
            std::process::exit(-1);
        }
        
        let Some(downloads) = dirs::download_dir() else {
            error!("Failed to find Downloads Folder");
            error_dialog("Failed to find Downloads folder.");
            std::process::exit(-1);
        };
        
        let Some(appdata) = dirs::data_local_dir() else {
            error!("Failed to find AppData/Local");
            error_dialog("Failed to find AppData/Local.");
            std::process::exit(-1);
        };

        let exe_path = match std::env::current_exe() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to find the path of the executable {}", e);
                error_dialog("Failed to find the path of the executable.");
                std::process::exit(-1);
            },
        };

        let Some(exe_folder_path) = exe_path.parent() else {
            error!("Failed to get the path to the folder containing the installer");
            error_dialog("Failed to get the path to the folder containing the installer");
            std::process::exit(-1);
        };

        // Non fatal
        let active_language = match get_user_language() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to get active language {e}");
                Language::English
            },
        };

        let appdata_path = PathBuf::from(&appdata);

        let mut downloader_path = appdata_path.clone();
        downloader_path.push("YTDownloader");

        let mut yt_dlp_path = downloader_path.clone();
        yt_dlp_path.push("yt-dlp.exe");

        let ffmpeg_path = downloader_path.clone();
        let ffprobe_path = downloader_path.clone();
        let deno_path = downloader_path.clone();

        let active_screen = if exe_folder_path == downloader_path {
            Screen::Home
        } else {
            Screen::Install
        };
      
        Self {
            languages: TextDatabase::default(),
            active_language,

            downloads_path: PathBuf::from(&downloads),
            appdata_path,
            downloader_path,

            active_screen,
        }
    }
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        container("HEllo").into()
    }

    fn translation(&self) -> &Translation {
        self.languages.get_translation(self.active_language)
    }
}

#[derive(Clone)]
enum Message {
}

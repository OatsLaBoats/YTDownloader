#![windows_subsystem = "windows"]

use std::fmt::Debug;
use std::path::PathBuf;

use iced::*;
use iced::widget::*;
use iced::widget::column;

use yt_downloader::lang::*;
use yt_downloader::screen::*;
use yt_downloader::platform::windows::*;

use tracing::error;

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

        let appdata_path = PathBuf::from(&appdata);

        let mut downloader_path = appdata_path.clone();
        downloader_path.push("YTDownloader");

        let mut yt_dlp_path = downloader_path.clone();
        yt_dlp_path.push("yt-dlp.exe");

        let ffmpeg_path = downloader_path.clone();
        let ffprobe_path = downloader_path.clone();
        let deno_path = downloader_path.clone();

        let active_screen = Screen::Update;
       
        Self {
            languages: TextDatabase::default(),
            active_language: Language::English,

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
        container("").into()
    }

    fn translation(&self) -> &Translation {
        self.languages.get_translation(self.active_language)
    }
}

async fn routine() {
    tokio::process::Command::new(which::which("powershell").unwrap())
        .arg("-c")
        .arg("Wait-Process -Name yt_downloader; $WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut(\"$Home\\Desktop\\YT Downloader.lnk\"); $Shortcut.TargetPath = \"$Home\\Dev\\Projects\\yt-downloader\\target\\debug\\yt_downloader.exe\"; $Shortcut.Save(); ")
        .spawn()
        .expect("Failed to spawn")
        .wait()
        .await.unwrap();
}

#[derive(Clone)]
enum Message {
}

#![windows_subsystem = "windows"]

use std::path::PathBuf;

use iced::*;
use iced::widget::*;
use iced::widget::column;

use yt_downloader::lang::*;
use yt_downloader::screen;
use yt_downloader::platform::windows::*;

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
}

impl Default for App {
    fn default() -> Self {
        // TODO: Handle errors
        let downloads = dirs::download_dir().unwrap();
        let appdata = dirs::data_local_dir().unwrap();

        let appdata_path = PathBuf::from(&appdata);

        let mut downloader_path = appdata_path.clone();
        downloader_path.push("YTDownloader");

        let mut yt_dlp_path = downloader_path.clone();
        yt_dlp_path.push("yt-dlp.exe");

        let ffmpeg_path = downloader_path.clone();
        let ffprobe_path = downloader_path.clone();
        let deno_path = downloader_path.clone();
       
        Self {
            languages: TextDatabase::default(),
            active_language: Language::English,

            downloads_path: PathBuf::from(&downloads),
            appdata_path,
            downloader_path,
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

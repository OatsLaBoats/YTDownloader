#![windows_subsystem = "console"]

use std::sync::Arc;

use tracing::error;
use iced::{Element, Task};

use yt_downloader::screen::update::UpdateKind;
use yt_downloader::*;
use yt_downloader::lang::*;
use yt_downloader::screen::Screen;
use yt_downloader::screen;
use yt_downloader::platform::windows::*;

// TODO: Write the settings file out when settings change
// TODO: Check versions of tools and download from github
 
// TODO: Redo error handling after the app if done, Remove anyhow

fn main() -> iced::Result {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subsriber failed");

    let icon_data = include_bytes!("../res/YTDownloader.png");
    let icon = iced::window::icon::from_file_data(icon_data, None).ok();
    
    iced::application(App::new, App::update, App::view)
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
    paths: Arc<Paths>,
    settings: yt_downloader::Settings,
    active_screen: Screen,

    http_client: reqwest::Client,
}

impl App {
    // NOTE: During initialization all errors are fatal. We need to make sure they are kept to a minimum.
    fn new() -> (Self, Task<Message>) {
        // Ensure we can run the install script. In the future maybe support other ways like bash or cmd.
        if let Err(e) = which::which("powershell") {
            error!("Failed to find powershell executable {}", e);
            error_dialog("Failed to find powershell executable");
            std::process::exit(-1);
        }
        
        let Some(downloads_dir) = dirs::download_dir() else {
            error!("Failed to find Downloads Folder");
            error_dialog("Failed to find Downloads folder.");
            std::process::exit(-1);
        };
        
        let Some(appdata_dir) = dirs::data_local_dir() else {
            error!("Failed to find AppData/Local");
            error_dialog("Failed to find AppData/Local.");
            std::process::exit(-1);
        };

        let Some(desktop_dir_path) = dirs::desktop_dir() else {
            error!("Failed to find Desktop directory");
            error_dialog("Failed to find Desktop directory");
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

        let Some(exe_dir_path) = exe_path.parent() else {
            error!("Failed to get the path to the folder containing the installer");
            error_dialog("Failed to get the path to the folder containing the installer");
            std::process::exit(-1);
        };

        
        let http_client = match reqwest::Client::builder().build() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to initialize HTTP client {e}");
                error_dialog("Failed to initialize HTTP client");
                std::process::exit(-1);
            },
        };

        // Non fatal
        let user_language = match get_user_language() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to get active language {e}");
                Language::English
            },
        };

        let mut downloader_dir = appdata_dir.clone();
        downloader_dir.push("YTDownloader");

        let mut bin_dir = downloader_dir.clone();
        bin_dir.push("bin");

        let mut yt_dlp_exe = bin_dir.clone();
        yt_dlp_exe.push("yt-dlp.exe");

        let mut ffmpeg_dir = bin_dir.clone();
        ffmpeg_dir.push("ffmpeg");

        let mut deno_exe = bin_dir.clone();
        deno_exe.push("deno.exe");

        let mut settings_file = downloader_dir.clone();
        settings_file.push("settings.json");

        let mut old_yt_downloader_exe = desktop_dir_path.clone();
        old_yt_downloader_exe.push("YTDownloader.exe");

        let mut old_yt_dlp_exe = downloader_dir.clone();
        old_yt_dlp_exe.push("yt-dlp.exe");

        let mut old_ffmpeg_exe = downloader_dir.clone();
        old_ffmpeg_exe.push("ffmpeg.exe");

        let mut old_deno_exe = downloader_dir.clone();
        old_deno_exe.push("deno.exe");

        let mut old_version_file = downloader_dir.clone();
        old_version_file.push("version");

        let paths = Arc::new(Paths {
            downloads_dir,
            appdata_dir,
            downloader_dir,
            bin_dir,
            yt_dlp_exe,
            ffmpeg_dir,
            deno_exe,
            settings_file,

            old_yt_downloader_exe,
            old_yt_dlp_exe,
            old_ffmpeg_exe,
            old_deno_exe,
            old_version_file,
        });


        let mut settings = Settings {
            ui_language: user_language,
            ui_theme: Theme::Auto,
        };

        // Read the settings file if they exist
        if paths.settings_file.exists() {
            let mut had_error = false;
            let contents = match std::fs::read_to_string(&paths.settings_file) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to read settings file {e}");
                    had_error = true;
                    String::new()
                },
            };

            if !had_error {
                match serde_json::from_str(&contents) {
                    Ok(v) => settings = v,
                    Err(e) => {
                        error!("Failed to parse settings file {e}");
                    },
                }
            }
        }

        tracing::info!("{settings:?}");

        let mut languages = TextDatabase::default();
        languages.current_language = settings.ui_language;
        
        let active_screen;
        let task;

        // Set the default screen and launch the start task to begin updating
        if exe_dir_path != paths.downloader_dir {
            let sc = screen::update::Screen::new(Arc::clone(&paths));
            task = sc.start(UpdateKind::Install, &http_client).map(Message::UpdateScreenMessage);
            active_screen = Screen::Update(sc);
        } else {
            active_screen = Screen::Home;
            task = Task::none();
        }

        (
            Self {
                languages,
                paths,
                settings,
                active_screen,
                http_client,
            },

            task,
        )
    }    

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateScreenMessage(message) => {
                if let Screen::Update(update_screen) = &mut self.active_screen {
                    let action = update_screen.update(message);
                    match action {
                        screen::update::Action::None => Task::none(),
                        screen::update::Action::Run(task) => task.map(Message::UpdateScreenMessage),
                    }
                } else {
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.active_screen {
            Screen::Update(update_screen) => {
                update_screen.view(self.languages.translation())
                    .map(Message::UpdateScreenMessage)
            },

            Screen::Home => {
                todo!()
            },
        }
    }
}

#[derive(Clone)]
enum Message {
    UpdateScreenMessage(screen::update::Message),
}

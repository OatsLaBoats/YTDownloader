#![windows_subsystem = "console"]

use std::io::Write;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tracing::{error, info};
use iced::{Element, Task};

use yt_downloader::*;
use yt_downloader::lang::*;
use yt_downloader::screen::Screen;
use yt_downloader::screen::update::UpdateKind;
use yt_downloader::screen;
use yt_downloader::platform::windows::*;

// TODO: Write the settings file out when settings change
// TODO: Settings migration mechanism for when settings change
// TODO: Check for main executable update
// TODO: Imporve logging with tags
// TODO: Add tooltips
// TODO: Maybe log the errors inside the async functions instead of returning them outside

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
        .theme(App::theme)
        .run()
}

#[derive(Default)]
struct App(Option<State>);

// Wrapper around state because it might fail on initialization
impl App {
    fn new() -> (Self, Task<Message>) {
        match State::new() {
            Some((s, t)) => (App(Some(s)), t),
            None => (App(None), iced::exit()),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match &mut self.0 {
            Some(v) => v.update(message),
            None => iced::exit(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.0 {
            Some(v) => v.view(),
            None => iced::widget::container("").into(),
        }
    }

    fn theme(&self) -> iced::Theme {
        match &self.0 {
            Some(v) => v.theme(),
            None => iced::Theme::Dark,
        }
    }
}
 
// Holds the state for the whole app.
struct State {
    languages: TextDatabase,
    paths: Arc<Paths>,
    settings: yt_downloader::Settings,
    active_screen: Screen,
    http_client: reqwest::Client,
    default_theme: iced::Theme,
}

impl State {
    // NOTE: During initialization all errors are fatal. We need to make sure they are kept to a minimum.
    fn new() -> Option<(Self, Task<Message>)> {
        // Ensure we can run the install script. In the future maybe support other ways like bash or cmd.
        if let Err(e) = which::which("powershell") {
            error!("Failed to find powershell executable {}", e);
            error_dialog("Failed to find powershell executable");
            return None;
        }
        
        let Some(downloads_dir) = dirs::download_dir() else {
            error!("Failed to find Downloads Folder");
            error_dialog("Failed to find Downloads folder.");
            return None;
        };
        
        let Some(appdata_dir) = dirs::data_local_dir() else {
            error!("Failed to find AppData/Local");
            error_dialog("Failed to find AppData/Local.");
            return None;
        };

        let Some(desktop_dir_path) = dirs::desktop_dir() else {
            error!("Failed to find Desktop directory");
            error_dialog("Failed to find Desktop directory");
            return None;
        };

        let exe_path = match std::env::current_exe() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to find the path of the executable {}", e);
                error_dialog("Failed to find the path of the executable.");
                return None;
            },
        };

        let Some(exe_dir_path) = exe_path.parent() else {
            error!("Failed to get the path to the folder containing the installer");
            error_dialog("Failed to get the path to the folder containing the installer");
            return None;
        };

        
        let http_client = match reqwest::Client::builder().build() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to initialize HTTP client {e}");
                error_dialog("Failed to initialize HTTP client");
                return None;
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
        downloader_dir.push("YT Downloader");

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
            auto_updates: true,
            ui_language: user_language,
            ui_theme: Theme::Auto,
            download_dir: paths.downloads_dir.to_string_lossy().to_string(),
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
        } else if paths.downloader_dir.exists() {
            match serde_json::to_string(&settings) {
                Ok(v) => {
                    match std::fs::File::create(&paths.settings_file) {
                        Ok(mut file) => {
                            let _ = file.write_all(v.as_bytes())
                                .map_err(|e| error!("Failed to write settings file {e}"));
                        },
                        Err(e) => error!("Failed to create settings.json {e}"),
                    }
                },
                Err(e) => error!("Failed to convert settings to json {e}"),
            }
        }

        let default_theme = match get_user_theme() {
            Ok(v) => {
                info!("Default theme {v:?}");
                v
            },
            Err(e) => {
                error!("Failed to find default theme {e}");
                iced::Theme::Dark
            },
        };

        let mut languages = TextDatabase::default();
        languages.current_language = settings.ui_language;
        
        let active_screen;
        let task;

        // Set the default screen and launch the start task to begin updating
        #[cfg(not(debug_assertions))]
        if exe_dir_path != paths.downloader_dir {
            let mut sc = screen::update::Screen::new(Arc::clone(&paths));
            task = sc.start(UpdateKind::Install, &http_client).map(Message::UpdateScreenMessage);
            active_screen = Screen::Update(sc);
        } else {
            let mut sc = screen::home::Screen::new(Arc::clone(&paths), settings.clone());
            task = sc.start(&http_client).map(Message::HomeScreenMessage);
            active_screen = Screen::Home(sc);
        }

        #[cfg(debug_assertions)]
        {
            let mut sc = screen::home::Screen::new(Arc::clone(&paths), settings.clone());
            task = sc.start(&http_client).map(Message::HomeScreenMessage);
            active_screen = Screen::Home(sc);
        }

        Some((
            Self {
                languages,
                paths,
                settings,
                active_screen,
                http_client,
                default_theme,
            },

            task,
        ))
    }    

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SaveSettings(_) => Task::none(),
            
            Message::UpdateScreenMessage(message) => {
                if let Screen::Update(update_screen) = &mut self.active_screen {
                    let action = update_screen.update(message);
                    match action {
                        screen::update::Action::Done(None) => iced::exit(),
                        screen::update::Action::Exit => iced::exit(),
                        screen::update::Action::Done(Some(task)) => task.map(Message::UpdateScreenMessage),
                        screen::update::Action::None => Task::none(),
                    }
                } else {
                    Task::none()
                }
            },

            Message::HomeScreenMessage(message) => {
                if let Screen::Home(home_screen) = &mut self.active_screen {
                    let action = home_screen.update(message);
                    match action {
                        screen::home::Action::UpdateNeeded => {
                            let mut sc = screen::update::Screen::new(Arc::clone(&self.paths));
                            let task = sc.start(UpdateKind::Normal, &self.http_client);
                            self.active_screen = Screen::Update(sc);
                            task.map(Message::UpdateScreenMessage)
                        },

                        screen::home::Action::SettingsChanged(settings) => {
                            self.settings = settings.clone();
                            Task::perform(
                                save_settings(
                                    Arc::clone(&self.paths),
                                    settings,
                                ),
                                Message::SaveSettings,
                            )
                        },

                        screen::home::Action::None => Task::none(),
                    }
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.active_screen {
            Screen::Update(update_screen) => {
                update_screen.view(self.languages.translation())
                    .map(Message::UpdateScreenMessage)
            },

            Screen::Home(home_screen) => {
                home_screen.view(self.languages.translation())
                    .map(Message::HomeScreenMessage)
            },
        }
    }

    fn theme(&self) -> iced::Theme {
        match self.settings.ui_theme {
            Theme::Dark => iced::Theme::Dark,
            Theme::Light => iced::Theme::CatppuccinLatte,
            Theme::Auto => self.default_theme.clone(),
        }
    }
}

async fn save_settings(paths: Arc<Paths>, settings: Settings) {
    let Ok(json) = serde_json::to_string(&settings) else {
        info!("SAVE_SETTINGS: failed to serialize settings");
        return;
    };

    if paths.settings_file.exists() {
        match tokio::fs::remove_file(&paths.settings_file).await {
            Ok(_) => {},
            Err(e) => {
                info!("SAVE_SETTINGS: failed to delete \"settings.json\" -> {e}");
                return;
            },
        }
    }
    
    let mut file = match tokio::fs::File::create(&paths.settings_file).await {
        Ok(v) => v,
        Err(e) => {
            info!("SAVE_SETTINGS: failed to create \"settings.json\" -> {e}");
            return;
        },
    };

    match file.write_all(json.as_bytes()).await {
        Ok(_) => {},
        Err(e) => {
            error!("SAVE_SETTINGS: failed to write \"settings.json\" -> {e}");
        },
    }
}

#[derive(Clone, Debug)]
enum Message {
    UpdateScreenMessage(screen::update::Message),
    HomeScreenMessage(screen::home::Message),
    SaveSettings(()),
}

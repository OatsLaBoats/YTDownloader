use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub mod platform;
pub mod screen;
pub mod lang;
pub mod github;
pub mod command;
pub mod widget;

use lang::Language;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,

    #[default]
    Auto,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Auto => "Auto",
        };

        write!(f, "{s}")
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub auto_updates: bool,
    pub ui_language: Language,
    pub ui_theme: Theme,
    pub download_dir: String,
}

#[derive(Default)]
pub struct Paths {
    pub downloads_dir: PathBuf,
    pub appdata_dir: PathBuf,
    pub downloader_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub yt_dlp_exe: PathBuf,
    pub ffmpeg_dir: PathBuf,
    pub deno_exe: PathBuf,
    pub settings_file: PathBuf,

    pub old_yt_downloader_exe: PathBuf,
    pub old_yt_dlp_exe: PathBuf,
    pub old_ffmpeg_exe: PathBuf,
    pub old_deno_exe: PathBuf,
    pub old_version_file: PathBuf,
}

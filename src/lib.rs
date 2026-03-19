use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub mod platform;
pub mod screen;
pub mod lang;
pub mod github;
pub mod command;

use lang::Language;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Theme {
    Dark,
    Light,
    Auto,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub ui_language: Language,
    pub ui_theme: Theme,
    pub download_dir: String,
}

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

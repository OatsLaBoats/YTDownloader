use std::path::PathBuf;

pub mod platform;
pub mod screen;
pub mod lang;

pub struct AppPaths {
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

// Utilities for windows

use std::ffi::{CString, OsString};
use std::os::windows::ffi::OsStringExt;
use std::str::FromStr;
use std::sync::Arc;

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Globalization::*;
use windows::core::{PCSTR, PWSTR, s};
use windows::Win32::UI::Shell::*;
use windows::Win32::System::Threading::*;
use thiserror::Error;

use crate::lang::Language;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("invocation to GetUserPreferredUILanguages failed")]
    GetUserPreferredLanguageFailed(Arc<windows::core::Error>),

    #[error("failed to convert OsString to rust String")]
    ConvertOsStringToUTF8Failed,

    #[error("failed to get the yt_downloader executable path")]
    GetExePathFailed(Arc<std::io::Error>),

    #[error("failed to spawn powershell instance")]
    SpawnPowershellCommandFailed(Arc<std::io::Error>),
    
    #[error("failed to open registry key")]
    OpenRegistryKeyFailed(windows_result::Error),

    #[error("failed to query the theme from the registry")]
    QueryThemeFailed(windows_result::Error),

    #[error("failed to convert rust string to c string")]
    ConvertRustStringToCStringFailed,

    #[error("failed to open process")]
    OpenProcessFailed(Arc<windows::core::Error>),

    #[error("failed to terminate process")]
    TermianteProcessFailed(Arc<windows::core::Error>),
}

pub fn kill_process(id: u32) -> Result<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, id)
            .map_err(|e| Error::OpenProcessFailed(Arc::new(e)))?;

        TerminateProcess(handle, 1)
            .map_err(|e| Error::TermianteProcessFailed(Arc::new(e)))?;

        let _ = CloseHandle(handle);
    }

    Ok(())
}

pub fn open_file_explorer(dir: &str) -> Result<()> {
    let d = CString::from_str(dir).map_err(|_| Error::ConvertRustStringToCStringFailed)?;

    unsafe {
        ShellExecuteA(None, s!("open"), PCSTR(d.as_ptr().cast()), PCSTR::null(), PCSTR::null(), SW_SHOW);
    }

    Ok(())
}

pub fn get_user_theme() -> Result<iced::Theme> {
    let key = windows_registry::CURRENT_USER
        .open(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize").map_err(
            Error::OpenRegistryKeyFailed
        )?;

    let light_theme = key
        .get_u32("AppsUseLightTheme").map_err(
            Error::QueryThemeFailed
        )?;

    if light_theme == 1 {
        Ok(iced::Theme::Light)
    } else {
        Ok(iced::Theme::Dark)
    }
}

pub fn get_user_language() -> Result<Language> {
    unsafe {
        let mut n_languages = 0u32;
        let mut buf_size = 0u32;

        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut n_languages, None, &mut buf_size).map_err(|e|
            Error::GetUserPreferredLanguageFailed(Arc::new(e))
        )?;

        let mut buf = Vec::with_capacity(buf_size as usize);
        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut n_languages, Some(PWSTR(buf.as_mut_ptr())), &mut buf_size).map_err(|e|
            Error::GetUserPreferredLanguageFailed(Arc::new(e))
        )?;

        buf.set_len(buf_size as usize);
        let names = OsString::from_wide(&buf).into_string().map_err(|_|
            Error::ConvertOsStringToUTF8Failed
        )?;

        if names.contains("de") {
            Ok(Language::German)
        } else {
            Ok(Language::English)
        }
    }
}

pub fn error_dialog(text: &str) {
    let message = CString::new(text).unwrap();
    unsafe {
        MessageBoxA(None, PCSTR(message.as_ptr().cast()), s!("Error"), MB_OK | MB_ICONERROR | MB_DEFAULT_DESKTOP_ONLY);
    }
}

// We use a powershell command to handle restarting the app:
// 1. Waits for the process to exit.
// 2. Moves the executable to it's AppData/Local location.
// 3. Creates a shortcut on the desktop.
// 4. Relaunch the app.
pub async fn finish_install_process(
    yt_dlp_updated: bool,
    ffmpeg_updated: bool,
    deno_updated: bool,
) -> Result<()> {
    let update_yt_dlp = if yt_dlp_updated {
        "Move-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\\yt-dlp.exe\" -Destination \"$env:LOCALAPPDATA\\YT Downloader\\bin\";"
    } else {
        ""
    };

    let update_ffmpeg = if ffmpeg_updated {
        "Move-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\\ffmpeg\" -Destination \"$env:LOCALAPPDATA\\YT Downloader\\bin\";"
    } else {
        ""
    };

    let update_deno = if deno_updated {
        "Move-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\\deno.exe\" -Destination \"$env:LOCALAPPDATA\\YT Downloader\\bin\";"
    } else {
        ""
    };
    
    let exe_path = std::env::current_exe().map_err(|e|
        Error::GetExePathFailed(Arc::new(e))
    )?;

    let exe = exe_path.into_os_string().into_string().map_err(|_|
        Error::ConvertOsStringToUTF8Failed
    )?;

    tokio::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW.0)
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(format!(" \
                Wait-Process -Name yt_downloader; \
                {update_yt_dlp}\
                {update_ffmpeg}\
                {update_deno}\
                Move-Item -Path {} -Destination \"$env:LOCALAPPDATA\\YT Downloader\\\"; \
                Remove-Item -Recurse -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\"; \
                $WshShell = New-Object -ComObject WScript.Shell; \
                $Shortcut = $WshShell.CreateShortcut(\"$HOME\\Desktop\\YT Downloader.lnk\"); \
                $Shortcut.TargetPath = \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
                $Shortcut.Save(); \
                Start-Process -FilePath \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
            ", exe))
        .spawn().map_err(|e|
            Error::SpawnPowershellCommandFailed(Arc::new(e))
        )?;

    Ok(())
}

// The only difference is the removal of the old executable
pub async fn finish_update_process(
    yt_dlp_updated: bool,
    ffmpeg_updated: bool,
    deno_updated: bool,
    app_updated: bool,
) -> Result<()> {
    let update_yt_dlp = if yt_dlp_updated {
        "Move-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\\yt-dlp.exe\" -Destination \"$env:LOCALAPPDATA\\YT Downloader\\bin\";"
    } else {
        ""
    };

    let update_ffmpeg = if ffmpeg_updated {
        "Move-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\\ffmpeg\" -Destination \"$env:LOCALAPPDATA\\YT Downloader\\bin\";"
    } else {
        ""
    };

    let update_deno = if deno_updated {
        "Move-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\\deno.exe\" -Destination \"$env:LOCALAPPDATA\\YT Downloader\\bin\";"
    } else {
        ""
    };

    let update_app = if app_updated {
        "Move-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\\yt_downloader.exe\" -Destination \"$env:LOCALAPPDATA\\YT Downloader\";"
    } else {
        ""
    };
    
    tokio::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW.0)
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(format!(" \
                Wait-Process -Name yt_downloader; \
                {update_yt_dlp}\
                {update_ffmpeg}\
                {update_deno}\
                Remove-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
                {update_app}\
                Remove-Item -Recurse -Path \"$env:LOCALAPPDATA\\YT Downloader\\downloads\"; \
                $WshShell = New-Object -ComObject WScript.Shell; \
                $Shortcut = $WshShell.CreateShortcut(\"$HOME\\Desktop\\YT Downloader.lnk\"); \
                $Shortcut.TargetPath = \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
                $Shortcut.Save(); \
                Start-Process -FilePath \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
        "))
        .spawn().map_err(|e|
            Error::SpawnPowershellCommandFailed(Arc::new(e))
        )?;

    Ok(())
}

pub async fn uninstall() -> Result<()> {
    tokio::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW.0)
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(" \
                Wait-Process -Name yt_downloader; \
                Remove-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\bin\\deno.exe\"; \
                Remove-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\bin\\yt-dlp.exe\"; \
                Remove-Item -Recurse -Path \"$env:LOCALAPPDATA\\YT Downloader\\bin\\ffmpeg\"; \
                Remove-Item -Recurse -Path \"$env:LOCALAPPDATA\\YT Downloader\\videos\"; \
                Remove-Item -Path \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
                Remove-Item -Path \"$HOME:Desktop\\YT Downloader.lnk\"; \
        ")
        .spawn().map_err(|e|
            Error::SpawnPowershellCommandFailed(Arc::new(e))
        )?;

    Ok(())
}

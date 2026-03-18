// Utilities for windows

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Globalization::*;
use windows::core::{PCSTR, PWSTR};

use crate::lang::Language;

pub fn get_user_language() -> anyhow::Result<Language> {
    unsafe {
        let mut n_languages = 0u32;
        let mut buf_size = 0u32;

        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut n_languages, None, &mut buf_size)?;

        let mut buf = Vec::with_capacity(buf_size as usize);
        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut n_languages, Some(PWSTR(buf.as_mut_ptr())), &mut buf_size)?;

        buf.set_len(buf_size as usize);
        let names = OsString::from_wide(&buf).into_string().unwrap(); // Should not fail at this point

        if names.contains("de") {
            Ok(Language::German)
        } else {
            Ok(Language::English)
        }
    }
}

pub fn error_dialog(text: &str) {
    unsafe {
        MessageBoxA(None, PCSTR(text.as_ptr()), PCSTR::null(), MB_OK | MB_ICONERROR);
    }
}

// We use a powershell command to handle restarting the app:
// 1. Waits for the process to exit.
// 2. Moves the executable to it's AppData/Local location.
// 3. Creates a shortcut on the desktop.
// 4. Relaunch the app.
pub async fn finish_update_process() -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe = exe_path.to_string_lossy();
    tokio::process::Command::new("powershell")
        .arg("-c")
        .arg(format!(" \
                Wait-Process -Name yt_downloader; \
                Move-Item -Path {} -Destination \"$env:LOCALAPPDATA\\YT Downloader\\\"; \
                $WshShell = New-Object -ComObject WScript.Shell; \
                $Shortcut = $WshShell.CreateShortcut(\"$HOME\\Desktop\\YT Downloader.lnk\"); \
                $Shortcut.TargetPath = \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
                $Shortcut.Save(); \
                Start-Process -FilePath \"$env:LOCALAPPDATA\\YT Downloader\\yt_downloader.exe\"; \
            ", exe))
        .spawn()?
        .wait().await?;
    Ok(())
}

// Utilities for windows

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

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_toolkit_path("C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.26100.0\\x64");
        res.set_icon("res/YTDownloader.ico");
        res.compile().unwrap();
    }
}

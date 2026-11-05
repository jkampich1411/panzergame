// Windows: Symbol und Versionsinfo in die panzer-launcher.exe einbetten
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icon/panzer.ico");
        res.set("ProductName", "Panzer");
        res.set("FileDescription", "Panzer Launcher");
        let _ = res.compile();
    }
}

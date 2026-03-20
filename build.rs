fn main() {
    // На Windows встраиваем иконку и манифест в .exe через winres
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        // Путь к .ico относительно корня проекта (рядом с Cargo.toml)
        res.set_icon("assets/icon.ico");
        // Название приложения в свойствах файла
        res.set("ProductName", "Trevailo Wallet");
        res.set("FileDescription", "Trevailo Coin GUI Wallet");
        res.set("LegalCopyright", "Trevailo Project");
        if let Err(e) = res.compile() {
            // Не прерываем сборку если winres недоступен — просто без иконки
            eprintln!("cargo:warning=winres failed: {e}");
        }
    }

    // Сообщаем Cargo: пересобирать если иконки изменились
    println!("cargo:rerun-if-changed=assets/icon.png");
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
mod adblock;
mod app;
mod browser;
mod commands;
mod config;
mod hints;
mod history;
mod keyboard;
mod mode;
mod statusbar;
mod tabs;

use gtk4::prelude::*;
use gtk4::Application;

const APP_ID: &str = "com.vimr.browser";

fn main() {
    // Fix white screen on many Linux systems — disables DMA-buf renderer
    // which often fails on Wayland/X11 without proper GPU setup
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = browser::BrowserWindow::new(app);
        window.present();
    });

    app.run();
}

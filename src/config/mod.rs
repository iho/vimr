pub mod lua_engine;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub homepage: String,
    pub show_urlbar: bool,
    pub show_statusbar: bool,
    pub adblock_enabled: bool,
    pub javascript_enabled: bool,
    pub dark_mode: bool,
    pub keybindings: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut kb = HashMap::new();
        kb.insert("j".into(), "scroll-down".into());
        kb.insert("k".into(), "scroll-up".into());
        kb.insert("h".into(), "scroll-left".into());
        kb.insert("l".into(), "scroll-right".into());
        kb.insert("ctrl+d".into(), "scroll-half-down".into());
        kb.insert("ctrl+u".into(), "scroll-half-up".into());
        kb.insert("ctrl+f".into(), "scroll-page-down".into());
        kb.insert("ctrl+b".into(), "scroll-page-up".into());
        kb.insert("gg".into(), "scroll-top".into());
        kb.insert("G".into(), "scroll-bottom".into());
        kb.insert("o".into(), "open-url".into());
        kb.insert("O".into(), "open-url-current".into());
        kb.insert("t".into(), "open-tab".into());
        kb.insert("r".into(), "reload".into());
        kb.insert("R".into(), "reload-hard".into());
        kb.insert("H".into(), "back".into());
        kb.insert("L".into(), "forward".into());
        kb.insert("f".into(), "hint-follow".into());
        kb.insert("F".into(), "hint-follow-new-tab".into());
        kb.insert("yy".into(), "yank-url".into());
        kb.insert("gt".into(), "tab-next".into());
        kb.insert("gT".into(), "tab-prev".into());
        kb.insert("g0".into(), "tab-first".into());
        kb.insert("g$".into(), "tab-last".into());
        kb.insert("i".into(), "insert-mode".into());
        kb.insert("Escape".into(), "normal-mode".into());
        kb.insert("zi".into(), "toggle-js".into());
        kb.insert("?".into(), "help".into());
        Config {
            homepage: "about:blank".into(),
            show_urlbar: true,
            show_statusbar: true,
            adblock_enabled: true,
            javascript_enabled: false,
            dark_mode: false,
            keybindings: kb,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("vimr"))
            .unwrap_or_default();

        let config_file = config_dir.join("config.lua");
        if config_file.exists() {
            match lua_engine::load_config(&config_file) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("Config error: {}", e),
            }
        }
        Config::default()
    }
}

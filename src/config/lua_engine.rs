use mlua::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use super::Config;

pub fn load_config(path: &Path) -> Result<Config, mlua::Error> {
    let lua = Lua::new();

    // Use Arc<Mutex<>> so the closures can be 'static
    let config: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::default()));

    {
        let config_clone = Arc::clone(&config);
        let set_fn = lua.create_function(move |_, (key, value): (String, mlua::Value)| {
            let mut cfg = config_clone.lock().unwrap();
            match key.as_str() {
                "homepage" => {
                    if let mlua::Value::String(s) = value {
                        cfg.homepage = s.to_str()?.to_string();
                    }
                }
                "ui.urlbar" => {
                    if let mlua::Value::Boolean(b) = value {
                        cfg.show_urlbar = b;
                    }
                }
                "ui.statusbar" => {
                    if let mlua::Value::Boolean(b) = value {
                        cfg.show_statusbar = b;
                    }
                }
                "adblock.enabled" => {
                    if let mlua::Value::Boolean(b) = value {
                        cfg.adblock_enabled = b;
                    }
                }
                "javascript.enabled" => {
                    if let mlua::Value::Boolean(b) = value {
                        cfg.javascript_enabled = b;
                    }
                }
                _ => {}
            }
            Ok(())
        })?;
        lua.globals().set("set", set_fn)?;
    }

    {
        let config_clone = Arc::clone(&config);
        let bind_fn = lua.create_function(move |_, (mode, key, action): (String, String, String)| {
            if mode == "normal" {
                config_clone.lock().unwrap().keybindings.insert(key, action);
            }
            Ok(())
        })?;
        lua.globals().set("bind", bind_fn)?;
    }

    let source = std::fs::read_to_string(path)
        .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
    lua.load(&source).exec()?;

    // Extract the config from Arc<Mutex<>>
    let result = Arc::try_unwrap(config)
        .map(|m| m.into_inner().unwrap())
        .unwrap_or_else(|arc| arc.lock().unwrap().clone());

    Ok(result)
}

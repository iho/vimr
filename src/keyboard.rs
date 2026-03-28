use gtk4::gdk;
use crate::app::AppState;

pub fn handle_normal_key(
    state: &mut AppState,
    keyval: gdk::Key,
    modifier: gdk::ModifierType,
) -> Option<String> {
    let key_str = key_to_string(keyval, modifier);

    if let Some(pending) = state.pending_key.take() {
        let combo = format!("{}{}", pending, key_str);
        if let Some(action) = state.config.keybindings.get(&combo) {
            return Some(action.clone());
        }
        // combo not found, fall through and try the current key alone
    }

    // If this key is a prefix of any multi-key binding, hold it and wait for the next key
    let is_prefix = state.config.keybindings.keys()
        .any(|k| k.len() > 1 && k.starts_with(key_str.as_str()));
    if is_prefix {
        state.pending_key = key_str.chars().next();
        return None;
    }

    if let Some(action) = state.config.keybindings.get(key_str.as_str()) {
        return Some(action.clone());
    }

    None
}

pub fn key_to_string(keyval: gdk::Key, modifier: gdk::ModifierType) -> String {
    let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);
    let shift = modifier.contains(gdk::ModifierType::SHIFT_MASK);

    let base = match keyval {
        gdk::Key::Escape => "Escape".to_string(),
        gdk::Key::Return => "Return".to_string(),
        gdk::Key::BackSpace => "BackSpace".to_string(),
        gdk::Key::Tab => "Tab".to_string(),
        gdk::Key::Up => "Up".to_string(),
        gdk::Key::Down => "Down".to_string(),
        gdk::Key::Left => "Left".to_string(),
        gdk::Key::Right => "Right".to_string(),
        _ => {
            if let Some(c) = keyval.to_unicode() {
                if shift && c.is_alphabetic() {
                    c.to_uppercase().to_string()
                } else {
                    c.to_string()
                }
            } else {
                return String::new();
            }
        }
    };

    if ctrl {
        format!("ctrl+{}", base.to_lowercase())
    } else {
        base
    }
}

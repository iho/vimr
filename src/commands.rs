pub enum CommandResult {
    OpenUrl(String),
    OpenTab(String),
    Reload,
    ReloadHard,
    Back,
    Forward,
    Quit,
    CloseTab,
    NewTab,
    ToggleJs,
    JsOn,
    JsOff,
    Help,
    AdblockUpdate,
    Unknown,
}

pub fn parse_command(input: &str) -> CommandResult {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    let cmd = parts[0];
    let arg = parts.get(1).copied().unwrap_or("");

    match cmd {
        "o" | "open" => CommandResult::OpenUrl(normalize_url(arg)),
        "t" | "tabopen" => CommandResult::OpenTab(normalize_url(arg)),
        "r" | "reload" => CommandResult::Reload,
        "R" | "reload!" => CommandResult::ReloadHard,
        "back" => CommandResult::Back,
        "forward" => CommandResult::Forward,
        "q" | "quit" => CommandResult::Quit,
        "bd" | "tabclose" => CommandResult::CloseTab,
        "tabnew" => CommandResult::NewTab,
        "js" => match arg {
            "on" | "1" | "true" => CommandResult::JsOn,
            "off" | "0" | "false" => CommandResult::JsOff,
            _ => CommandResult::ToggleJs,
        },
        "noscript" => CommandResult::JsOff,
        "help" => CommandResult::Help,
        "adblock-update" => CommandResult::AdblockUpdate,
        _ => CommandResult::Unknown,
    }
}

pub fn normalize_url(input: &str) -> String {
    if input.is_empty() {
        return "about:blank".to_string();
    }
    if input.starts_with("http://") || input.starts_with("https://")
        || input.starts_with("file://") || input.starts_with("about:")
    {
        return input.to_string();
    }
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{}", input);
    }
    format!("https://duckduckgo.com/?q={}", urlencoding_simple(input))
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

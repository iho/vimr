#[derive(Debug, Clone)]
pub struct TabInfo {
    pub url: String,
    pub title: String,
    pub is_loading: bool,
}

impl TabInfo {
    pub fn new(url: &str) -> Self {
        TabInfo {
            url: url.to_string(),
            title: String::new(),
            is_loading: false,
        }
    }
}

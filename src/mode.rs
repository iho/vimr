#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command(String),
    Hint { follow_new_tab: bool },
}

impl Mode {
    pub fn name(&self) -> &str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command(_) => "COMMAND",
            Mode::Hint { .. } => "HINT",
        }
    }

    pub fn is_passthrough(&self) -> bool {
        matches!(self, Mode::Insert)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

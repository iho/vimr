use std::cell::RefCell;
use std::rc::Rc;
use crate::mode::Mode;
use crate::config::Config;
use crate::adblock::manager::AdblockManager;

pub struct AppState {
    pub mode: Mode,
    pub config: Config,
    pub pending_key: Option<char>,
    pub adblock: Rc<AdblockManager>,
}

pub type SharedState = Rc<RefCell<AppState>>;

impl AppState {
    pub fn new() -> SharedState {
        let config = Config::load();
        let adblock = AdblockManager::new(config.adblock_enabled);
        Rc::new(RefCell::new(AppState {
            mode: Mode::Normal,
            config,
            pending_key: None,
            adblock,
        }))
    }
}

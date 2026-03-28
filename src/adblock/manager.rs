use adblock::{Engine, lists::{FilterSet, ParseOptions}};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

const FILTER_LISTS: &[(&str, &str)] = &[
    ("easylist.txt",    "https://easylist.to/easylist/easylist.txt"),
    ("easyprivacy.txt", "https://easylist.to/easylist/easyprivacy.txt"),
];

/// Adblock engine — GTK main thread only (adblock::Engine is !Send).
pub struct AdblockManager {
    pub enabled: bool,
    engine: Rc<RefCell<Option<Engine>>>,
}

impl AdblockManager {
    pub fn new(enabled: bool) -> Rc<Self> {
        let engine: Rc<RefCell<Option<Engine>>> = Rc::new(RefCell::new(None));
        let mgr = Rc::new(AdblockManager { enabled, engine: engine.clone() });

        if enabled {
            Self::start_load(engine, false);
        }

        mgr
    }

    /// Spawn background fetch then build engine on main thread via idle poll.
    fn start_load(engine: Rc<RefCell<Option<Engine>>>, force: bool) {
        // Arc<Mutex<>> for cross-thread handoff of the (Send) raw text
        let slot: Arc<Mutex<Option<Result<Vec<String>, String>>>> =
            Arc::new(Mutex::new(None));
        let writer = slot.clone();

        std::thread::spawn(move || {
            if force {
                purge_cache();
            }
            *writer.lock().unwrap() = Some(fetch_lists());
        });

        // Poll on GTK main thread (idle_add_local is !Send-friendly)
        glib::idle_add_local(move || {
            let mut guard = slot.lock().unwrap();
            if let Some(result) = guard.take() {
                match result {
                    Ok(contents) => {
                        let mut fs = FilterSet::new(true);
                        for c in &contents {
                            fs.add_filter_list(c, ParseOptions::default());
                        }
                        *engine.borrow_mut() = Some(Engine::from_filter_set(fs, true));
                        eprintln!("[adblock] engine ready ({} lists)", contents.len());
                    }
                    Err(e) => eprintln!("[adblock] load failed: {e}"),
                }
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        });
    }

    pub fn should_block(&self, url: &str, source_url: &str, resource_type: &str) -> bool {
        if !self.enabled {
            return false;
        }
        let guard = self.engine.borrow();
        let Some(engine) = guard.as_ref() else { return false };
        adblock::request::Request::new(url, source_url, resource_type)
            .map(|req| engine.check_network_request(&req).matched)
            .unwrap_or(false)
    }

    /// Delete cache and rebuild engine. Call via :adblock-update.
    pub fn update(self: &Rc<Self>) {
        Self::start_load(self.engine.clone(), true);
    }
}

fn cache_dir() -> PathBuf {
    let dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("vimr").join("adblock");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn purge_cache() {
    let cache = cache_dir();
    for (filename, _) in FILTER_LISTS {
        std::fs::remove_file(cache.join(filename)).ok();
    }
}

fn fetch_lists() -> Result<Vec<String>, String> {
    let cache = cache_dir();
    let mut contents = Vec::new();
    for (filename, url) in FILTER_LISTS {
        let path = cache.join(filename);
        let text = if path.exists() {
            eprintln!("[adblock] loading {filename} from cache");
            std::fs::read_to_string(&path).map_err(|e| e.to_string())?
        } else {
            eprintln!("[adblock] downloading {filename}…");
            let body = ureq::get(url).call()
                .map_err(|e| e.to_string())?
                .into_string()
                .map_err(|e| e.to_string())?;
            std::fs::write(&path, &body).map_err(|e| e.to_string())?;
            body
        };
        contents.push(text);
    }
    Ok(contents)
}

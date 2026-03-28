use std::cell::RefCell;
use std::rc::Rc;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Entry, Label,
    Notebook, Orientation,
};
use webkit6::prelude::*;
use webkit6::{WebView, Settings as WebSettings};

use crate::app::{AppState, SharedState};
use crate::commands::{parse_command, CommandResult, normalize_url};
use crate::hints::{HINT_JS, HINT_ACTIVATE_JS, HINT_CLEAR_JS, SCROLL_JS};
use crate::keyboard::{handle_normal_key, key_to_string};
use crate::mode::Mode;
use crate::statusbar::StatusBar;
use crate::history::History;

pub struct BrowserWindow {
    pub window: ApplicationWindow,
}

impl BrowserWindow {
    pub fn new(app: &Application) -> Self {
        let state = AppState::new();
        let history: Rc<RefCell<Option<History>>> = Rc::new(RefCell::new(History::open().ok()));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("vimr")
            .default_width(1280)
            .default_height(800)
            .build();

        // Apply CSS
        let css = gtk4::CssProvider::new();
        css.load_from_data(BROWSER_CSS);
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let root = GtkBox::new(Orientation::Vertical, 0);

        let urlbar = Entry::new();
        urlbar.set_placeholder_text(Some("Enter URL or search..."));
        urlbar.add_css_class("urlbar");

        let notebook = Notebook::new();
        notebook.set_hexpand(true);
        notebook.set_vexpand(true);
        notebook.set_show_tabs(true);
        notebook.set_scrollable(true);

        let statusbar = Rc::new(StatusBar::new());

        root.append(&urlbar);
        root.append(&notebook);
        root.append(&statusbar.widget);

        window.set_child(Some(&root));

        let notebook = Rc::new(notebook);
        let urlbar = Rc::new(urlbar);

        let homepage = state.borrow().config.homepage.clone();
        Self::create_tab(
            &notebook,
            &homepage,
            state.clone(),
            urlbar.clone(),
            statusbar.clone(),
            history.clone(),
        );

        // URL bar activation — do this before returning
        {
            let state = state.clone();
            let notebook = notebook.clone();
            let statusbar = statusbar.clone();

            urlbar.connect_activate(move |entry| {
                let text = entry.text().to_string();
                let url = normalize_url(&text);
                if let Some(wv) = Self::current_webview(&notebook) {
                    wv.load_uri(&url);
                }
                state.borrow_mut().mode = Mode::Normal;
                statusbar.update_mode(&state.borrow().mode);
            });
        }

        BrowserWindow { window }
    }

    /// Attach a vim key controller directly to a webview.
    /// Must be on the webview itself (not the window) so WebKitGTK doesn't swallow events first.
    fn attach_key_controller(
        webview: &WebView,
        notebook: Rc<Notebook>,
        state: SharedState,
        urlbar: Rc<Entry>,
        statusbar: Rc<StatusBar>,
        history: Rc<RefCell<Option<History>>>,
    ) {
        let key_ctrl = gtk4::EventControllerKey::new();
        // Capture phase: runs before WebKit processes the key as the target widget
        key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);

        key_ctrl.connect_key_pressed(move |_, keyval, _, modifier| {
                let current_mode = state.borrow().mode.clone();

                match &current_mode {
                    Mode::Normal => {
                        let key_str = key_to_string(keyval, modifier);

                        // Enter command mode
                        if key_str == ":" {
                            state.borrow_mut().mode = Mode::Command(String::new());
                            statusbar.update_mode(&state.borrow().mode);
                            return glib::Propagation::Stop;
                        }

                        let webview = Self::current_webview(&notebook);

                        let action = {
                            let mut s = state.borrow_mut();
                            handle_normal_key(&mut s, keyval, modifier)
                        };

                        if let Some(action) = action {
                            if let Some(wv) = &webview {
                                Self::execute_action(
                                    &action,
                                    wv,
                                    &notebook,
                                    state.clone(),
                                    urlbar.clone(),
                                    statusbar.clone(),
                                    history.clone(),
                                );
                            }
                        }

                        glib::Propagation::Stop
                    }

                    Mode::Insert => {
                        let key_str = key_to_string(keyval, modifier);
                        if key_str == "Escape" {
                            state.borrow_mut().mode = Mode::Normal;
                            statusbar.update_mode(&state.borrow().mode);
                            return glib::Propagation::Stop;
                        }
                        glib::Propagation::Proceed
                    }

                    Mode::Command(current_input) => {
                        let key_str = key_to_string(keyval, modifier);
                        let mut input = current_input.clone();

                        match key_str.as_str() {
                            "Escape" => {
                                state.borrow_mut().mode = Mode::Normal;
                                statusbar.update_mode(&state.borrow().mode);
                                let url = Self::current_webview(&notebook)
                                    .as_ref()
                                    .and_then(|wv| wv.uri())
                                    .map(|u| u.to_string())
                                    .unwrap_or_default();
                                statusbar.update_url(&url);
                            }
                            "Return" => {
                                let cmd = input.clone();
                                state.borrow_mut().mode = Mode::Normal;
                                statusbar.update_mode(&state.borrow().mode);

                                let result = parse_command(&cmd);
                                let webview = Self::current_webview(&notebook);
                                if let Some(wv) = &webview {
                                    match result {
                                        CommandResult::OpenUrl(url) => {
                                            wv.load_uri(&url);
                                            urlbar.set_text(&url);
                                        }
                                        CommandResult::OpenTab(url) => {
                                            Self::create_tab(
                                                &notebook,
                                                &url,
                                                state.clone(),
                                                urlbar.clone(),
                                                statusbar.clone(),
                                                history.clone(),
                                            );
                                        }
                                        CommandResult::Reload => {
                                            wv.reload();
                                        }
                                        CommandResult::ReloadHard => {
                                            wv.reload_bypass_cache();
                                        }
                                        CommandResult::Back => {
                                            wv.go_back();
                                        }
                                        CommandResult::Forward => {
                                            wv.go_forward();
                                        }
                                        CommandResult::Quit => {
                                            std::process::exit(0);
                                        }
                                        CommandResult::CloseTab => {
                                            let page = notebook.current_page();
                                            if notebook.n_pages() > 1 {
                                                notebook.remove_page(page);
                                            }
                                        }
                                        CommandResult::NewTab => {
                                            let hp = state.borrow().config.homepage.clone();
                                            Self::create_tab(
                                                &notebook,
                                                &hp,
                                                state.clone(),
                                                urlbar.clone(),
                                                statusbar.clone(),
                                                history.clone(),
                                            );
                                        }
                                        CommandResult::ToggleJs => {
                                            Self::set_js(wv, None, &statusbar);
                                        }
                                        CommandResult::JsOn => {
                                            Self::set_js(wv, Some(true), &statusbar);
                                        }
                                        CommandResult::JsOff => {
                                            Self::set_js(wv, Some(false), &statusbar);
                                        }
                                        CommandResult::Help => {
                                            wv.load_html(HELP_HTML, None);
                                        }
                                        CommandResult::AdblockUpdate => {
                                            state.borrow().adblock.update();
                                            statusbar.update_info("Adblock: updating...");
                                        }
                                        CommandResult::Unknown => {
                                            statusbar.update_info(&format!("Unknown: {}", cmd));
                                        }
                                    }
                                }
                            }
                            "BackSpace" => {
                                input.pop();
                                state.borrow_mut().mode = Mode::Command(input.clone());
                                statusbar.update_mode(&state.borrow().mode);
                            }
                            s if s.len() == 1 => {
                                input.push_str(s);
                                state.borrow_mut().mode = Mode::Command(input.clone());
                                statusbar.update_mode(&state.borrow().mode);
                            }
                            _ => {}
                        }

                        glib::Propagation::Stop
                    }

                    Mode::Hint { follow_new_tab: _ } => {
                        let key_str = key_to_string(keyval, modifier);
                        let webview = Self::current_webview(&notebook);

                        if key_str == "Escape" {
                            if let Some(wv) = &webview {
                                wv.evaluate_javascript(
                                    HINT_CLEAR_JS,
                                    Some("vimr"),
                                    None,
                                    None::<&gio::Cancellable>,
                                    |_| {},
                                );
                            }
                            state.borrow_mut().mode = Mode::Normal;
                            statusbar.update_mode(&state.borrow().mode);
                        } else if key_str.len() == 1
                            && key_str.chars().next().map_or(false, |c| c.is_alphanumeric())
                        {
                            let js = format!("({})({:?})", HINT_ACTIVATE_JS, key_str);
                            if let Some(wv) = &webview {
                                wv.evaluate_javascript(
                                    &js,
                                    Some("vimr"),
                                    None,
                                    None::<&gio::Cancellable>,
                                    |_| {},
                                );
                            }
                            state.borrow_mut().mode = Mode::Normal;
                            statusbar.update_mode(&state.borrow().mode);
                        }

                        glib::Propagation::Stop
                    }
                }
            });

        webview.add_controller(key_ctrl);
    }

    fn create_tab(
        notebook: &Rc<Notebook>,
        url: &str,
        state: SharedState,
        urlbar: Rc<Entry>,
        statusbar: Rc<StatusBar>,
        history: Rc<RefCell<Option<History>>>,
    ) -> WebView {
        let js_enabled = state.borrow().config.javascript_enabled;

        let settings = WebSettings::new();
        // Keep the JS engine alive (needed for our own evaluate_javascript scroll/hint calls).
        // Only block page-authored scripts via enable_javascript_markup.
        settings.set_enable_javascript(true);
        settings.set_enable_javascript_markup(js_enabled);
        settings.set_enable_developer_extras(true);
        // Memory / resource savings
        settings.set_hardware_acceleration_policy(webkit6::HardwareAccelerationPolicy::Never);
        settings.set_enable_page_cache(false);
        settings.set_enable_dns_prefetching(false);
        settings.set_media_playback_requires_user_gesture(true);
        settings.set_enable_media_stream(false);
        settings.set_enable_webaudio(false);
        settings.set_enable_webgl(false);
        settings.set_enable_media(false);
        settings.set_auto_load_images(true); // keep images on, can be toggled later

        let webview = WebView::new();
        webview.set_settings(&settings);
        webview.set_hexpand(true);
        webview.set_vexpand(true);
        webview.load_uri(url);

        let tab_label = Label::new(Some("New Tab"));

        notebook.append_page(&webview, Some(&tab_label));
        notebook.set_current_page(Some(notebook.n_pages() - 1));
        webview.show();

        // Connect URI change
        {
            let urlbar = urlbar.clone();
            let statusbar = statusbar.clone();

            webview.connect_uri_notify(move |wv| {
                if let Some(uri) = wv.uri() {
                    urlbar.set_text(uri.as_str());
                    statusbar.update_url(uri.as_str());
                }
            });
        }

        // Connect title change
        {
            let statusbar = statusbar.clone();
            let history = history.clone();
            let tab_label = tab_label.clone();

            webview.connect_title_notify(move |wv| {
                let title = wv
                    .title()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "Untitled".to_string());
                let short_title: String = title.chars().take(24).collect();
                tab_label.set_text(&short_title);
                statusbar.update_info(&title);

                if let Some(uri) = wv.uri() {
                    if let Some(hist) = history.borrow().as_ref() {
                        let _ = hist.add(uri.as_str(), &title);
                    }
                }
            });
        }

        // Connect loading state
        {
            let statusbar = statusbar.clone();
            webview.connect_is_loading_notify(move |wv| {
                if wv.is_loading() {
                    statusbar.update_info("Loading...");
                }
            });
        }

        // Adblock: intercept resource loading decisions
        {
            let adblock = state.borrow().adblock.clone();
            webview.connect_decide_policy(move |_wv, decision, decision_type| {
                use webkit6::prelude::PolicyDecisionExt;
                if decision_type == webkit6::PolicyDecisionType::Response {
                    if let Ok(resp_decision) = decision.clone().downcast::<webkit6::ResponsePolicyDecision>() {
                        if let Some(response) = resp_decision.response() {
                            if let Some(uri) = response.uri() {
                                // Use empty string as source — good enough for matching
                                if adblock.should_block(uri.as_str(), "", "other") {
                                    decision.ignore();
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            });
        }

        // Attach vim key handler directly to this webview
        Self::attach_key_controller(
            &webview,
            notebook.clone(),
            state.clone(),
            urlbar.clone(),
            statusbar.clone(),
            history.clone(),
        );

        webview
    }

    fn current_webview(notebook: &Notebook) -> Option<WebView> {
        let page = notebook.current_page()?;
        let widget = notebook.nth_page(Some(page))?;
        widget.downcast::<WebView>().ok()
    }

    fn execute_action(
        action: &str,
        webview: &WebView,
        notebook: &Rc<Notebook>,
        state: SharedState,
        urlbar: Rc<Entry>,
        statusbar: Rc<StatusBar>,
        history: Rc<RefCell<Option<History>>>,
    ) {
        match action {
            "scroll-down" => Self::scroll(webview, "down"),
            "scroll-up" => Self::scroll(webview, "up"),
            "scroll-left" => Self::scroll(webview, "left"),
            "scroll-right" => Self::scroll(webview, "right"),
            "scroll-half-down" => Self::scroll(webview, "half-down"),
            "scroll-half-up" => Self::scroll(webview, "half-up"),
            "scroll-page-down" => Self::scroll(webview, "page-down"),
            "scroll-page-up" => Self::scroll(webview, "page-up"),
            "scroll-top" => Self::scroll(webview, "top"),
            "scroll-bottom" => Self::scroll(webview, "bottom"),
            "insert-mode" => {
                state.borrow_mut().mode = Mode::Insert;
                statusbar.update_mode(&state.borrow().mode);
            }
            "normal-mode" => {
                state.borrow_mut().mode = Mode::Normal;
                statusbar.update_mode(&state.borrow().mode);
            }
            "open-url" => {
                state.borrow_mut().mode = Mode::Command("o ".to_string());
                statusbar.update_mode(&state.borrow().mode);
            }
            "open-url-current" => {
                let url = webview.uri().map(|u| u.to_string()).unwrap_or_default();
                state.borrow_mut().mode = Mode::Command(format!("o {}", url));
                statusbar.update_mode(&state.borrow().mode);
            }
            "open-tab" => {
                state.borrow_mut().mode = Mode::Command("t ".to_string());
                statusbar.update_mode(&state.borrow().mode);
            }
            "reload" => {
                webview.reload();
            }
            "reload-hard" => {
                webview.reload_bypass_cache();
            }
            "back" => {
                webview.go_back();
            }
            "forward" => {
                webview.go_forward();
            }
            "hint-follow" => {
                state.borrow_mut().mode = Mode::Hint { follow_new_tab: false };
                statusbar.update_mode(&state.borrow().mode);
                webview.evaluate_javascript(
                    HINT_JS,
                    Some("vimr"),
                    None,
                    None::<&gio::Cancellable>,
                    |_| {},
                );
            }
            "hint-follow-new-tab" => {
                state.borrow_mut().mode = Mode::Hint { follow_new_tab: true };
                statusbar.update_mode(&state.borrow().mode);
                webview.evaluate_javascript(
                    HINT_JS,
                    Some("vimr"),
                    None,
                    None::<&gio::Cancellable>,
                    |_| {},
                );
            }
            "yank-url" => {
                if let Some(uri) = webview.uri() {
                    let display = gtk4::gdk::Display::default().unwrap();
                    let clipboard = display.clipboard();
                    clipboard.set_text(uri.as_str());
                    statusbar.update_info(&format!("Yanked: {}", uri));
                }
            }
            "tab-next" => {
                let pages = notebook.n_pages();
                if pages > 0 {
                    let cur = notebook.current_page().unwrap_or(0);
                    notebook.set_current_page(Some((cur + 1) % pages));
                }
            }
            "tab-prev" => {
                let pages = notebook.n_pages();
                if pages > 0 {
                    let cur = notebook.current_page().unwrap_or(0);
                    notebook.set_current_page(Some(if cur == 0 { pages - 1 } else { cur - 1 }));
                }
            }
            "tab-first" => {
                notebook.set_current_page(Some(0));
            }
            "tab-last" => {
                let pages = notebook.n_pages();
                if pages > 0 {
                    notebook.set_current_page(Some(pages - 1));
                }
            }
            "toggle-js" => {
                Self::set_js(webview, None, &statusbar);
            }
            "help" => {
                webview.load_html(HELP_HTML, None);
            }
            _ => {}
        }
    }

    /// Toggle or set JS on a webview. Pass None to toggle, Some(true/false) to force.
    fn set_js(webview: &WebView, enabled: Option<bool>, statusbar: &StatusBar) {
        use webkit6::prelude::WebViewExt;
        if let Some(settings) = webkit6::prelude::WebViewExt::settings(webview) {
            // Only toggle page script markup — never disable the JS engine itself
            // (we need it for our own evaluate_javascript scroll/hint calls).
            let new_state = enabled.unwrap_or(!settings.enables_javascript_markup());
            settings.set_enable_javascript_markup(new_state);
            statusbar.update_info(if new_state { "page JS: ON" } else { "page JS: OFF" });
        }
    }

    fn scroll(webview: &WebView, direction: &str) {
        let js = format!("({})({:?})", SCROLL_JS, direction);
        // "vimr" world: runs regardless of page JS enabled/disabled setting
        webview.evaluate_javascript(&js, Some("vimr"), None, None::<&gio::Cancellable>, |_| {});
    }

    pub fn present(&self) {
        self.window.present();
    }
}

const HELP_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>vimr help</title>
<style>
  body { font-family: monospace; background: #1a1a1a; color: #e0e0e0; padding: 2em; max-width: 700px; }
  h1 { color: #7ec8e3; margin-bottom: 0.5em; }
  h2 { color: #aaa; border-bottom: 1px solid #444; padding-bottom: 0.2em; }
  table { border-collapse: collapse; width: 100%; margin-bottom: 1.5em; }
  td { padding: 3px 12px 3px 0; vertical-align: top; }
  td:first-child { color: #ffcc00; white-space: nowrap; min-width: 120px; }
  tr:hover td { background: #2a2a2a; }
</style>
</head>
<body>
<h1>vimr — keyboard reference</h1>

<h2>Modes</h2>
<table>
  <tr><td>Escape</td><td>→ Normal mode (from any mode)</td></tr>
  <tr><td>i</td><td>→ Insert mode (keys pass to page)</td></tr>
  <tr><td>:</td><td>→ Command mode</td></tr>
  <tr><td>f / F</td><td>→ Hint mode (follow link / open in new tab)</td></tr>
</table>

<h2>Scrolling (Normal mode)</h2>
<table>
  <tr><td>j / k</td><td>Scroll down / up</td></tr>
  <tr><td>h / l</td><td>Scroll left / right</td></tr>
  <tr><td>Ctrl+d / Ctrl+u</td><td>Half page down / up</td></tr>
  <tr><td>Ctrl+f / Ctrl+b</td><td>Full page down / up</td></tr>
  <tr><td>gg / G</td><td>Top / bottom of page</td></tr>
</table>

<h2>Navigation (Normal mode)</h2>
<table>
  <tr><td>H / L</td><td>Back / forward</td></tr>
  <tr><td>r / R</td><td>Reload / hard reload</td></tr>
  <tr><td>o</td><td>Open URL or search</td></tr>
  <tr><td>O</td><td>Open current URL for editing</td></tr>
  <tr><td>yy</td><td>Yank (copy) current URL</td></tr>
</table>

<h2>Tabs (Normal mode)</h2>
<table>
  <tr><td>t</td><td>Open new tab</td></tr>
  <tr><td>gt / gT</td><td>Next / previous tab</td></tr>
  <tr><td>g0 / g$</td><td>First / last tab</td></tr>
</table>

<h2>JavaScript</h2>
<table>
  <tr><td>zi</td><td>Toggle JavaScript on/off</td></tr>
  <tr><td>:js</td><td>Toggle JavaScript</td></tr>
  <tr><td>:js on / :js off</td><td>Enable / disable JavaScript</td></tr>
  <tr><td>:noscript</td><td>Disable JavaScript</td></tr>
</table>

<h2>Commands (:)</h2>
<table>
  <tr><td>:o &lt;url&gt;</td><td>Open URL or search in current tab</td></tr>
  <tr><td>:t &lt;url&gt;</td><td>Open URL in new tab</td></tr>
  <tr><td>:r / :R</td><td>Reload / hard reload</td></tr>
  <tr><td>:q</td><td>Quit</td></tr>
  <tr><td>:bd</td><td>Close current tab</td></tr>
  <tr><td>:tabnew</td><td>New empty tab</td></tr>
  <tr><td>:back / :forward</td><td>Navigate history</td></tr>
  <tr><td>:help</td><td>This page</td></tr>
</table>

<h2>Hints (f mode)</h2>
<table>
  <tr><td>f</td><td>Show hints, type letters to follow link</td></tr>
  <tr><td>F</td><td>Show hints, open in new tab</td></tr>
  <tr><td>Escape</td><td>Cancel hints</td></tr>
</table>
</body></html>
"#;

const BROWSER_CSS: &str = r#"
window, .browser-window {
    background: #1a1a1a;
}
.urlbar {
    background: #2a2a2a;
    color: #e0e0e0;
    border: none;
    border-radius: 0;
    font-family: monospace;
    font-size: 13px;
    padding: 4px 8px;
    min-height: 24px;
}
.statusbar {
    background: #2a2a2a;
    color: #e0e0e0;
    font-family: monospace;
    font-size: 12px;
    padding: 2px 8px;
    min-height: 20px;
    border-top: 1px solid #444;
}
.mode-label {
    color: #7ec8e3;
    font-weight: bold;
}
.mode-label.insert {
    color: #90ee90;
}
"#;

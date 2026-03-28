use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use crate::mode::Mode;

pub struct StatusBar {
    pub widget: GtkBox,
    mode_label: Label,
    url_label: Label,
    info_label: Label,
}

impl StatusBar {
    pub fn new() -> Self {
        let widget = GtkBox::new(Orientation::Horizontal, 8);
        widget.add_css_class("statusbar");

        let mode_label = Label::new(Some("NORMAL"));
        mode_label.add_css_class("mode-label");
        mode_label.set_width_chars(8);
        mode_label.set_xalign(0.0);

        let url_label = Label::new(Some(""));
        url_label.set_hexpand(true);
        url_label.set_xalign(0.0);
        url_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        let info_label = Label::new(Some(""));
        info_label.set_xalign(1.0);

        widget.append(&mode_label);
        widget.append(&url_label);
        widget.append(&info_label);

        StatusBar { widget, mode_label, url_label, info_label }
    }

    pub fn update_mode(&self, mode: &Mode) {
        self.mode_label.set_text(mode.name());
        match mode {
            Mode::Normal => self.mode_label.remove_css_class("insert"),
            Mode::Insert => self.mode_label.add_css_class("insert"),
            Mode::Command(cmd) => {
                self.url_label.set_text(&format!(":{}", cmd));
            }
            Mode::Hint { .. } => {}
        }
    }

    pub fn update_url(&self, url: &str) {
        self.url_label.set_text(url);
    }

    pub fn update_info(&self, text: &str) {
        self.info_label.set_text(text);
    }
}

use gtk::{builders::BoxBuilder, prelude::*, Box as GtkBox, Button, Entry, Orientation, TextView};

pub struct Footer {
    pub inner: GtkBox,
}

impl Footer {
    pub fn new() -> Self {
        let inner = BoxBuilder::new()
            .orientation(Orientation::Horizontal)
            .margin_start(4)
            .margin_end(4)
            .hexpand(true)
            .build();
        let entry = TextView::new();
        entry.set_hexpand(true);
        entry.set_monospace(true);

        let send = Button::from_icon_name("media-record-symbolic");
        inner.append(&entry);
        inner.append(&send);

        Self { inner }
    }
}

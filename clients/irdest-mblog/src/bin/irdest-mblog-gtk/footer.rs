use gtk::{builders::BoxBuilder, prelude::*, Box as GtkBox, Button, Entry, Orientation};

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
        let entry = Entry::new();
        entry.set_hexpand(true);
        let send = Button::from_icon_name("send-symbolic");
        inner.append(&entry);
        inner.append(&send);

        Self { inner }
    }
}

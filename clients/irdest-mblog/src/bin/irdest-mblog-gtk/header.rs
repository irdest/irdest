use gtk::prelude::*;
use gtk::{Button, HeaderBar, Label};

pub struct Header {
    pub inner: HeaderBar,
    add_topic: Button,
}

impl Header {
    pub fn new() -> Header {
        let inner = HeaderBar::new();
        inner.set_show_title_buttons(true);
        inner.set_title_widget(Some(&Label::new(Some("Irdest mblog"))));

        let add_topic = Button::from_icon_name("folder-new-symbolic");
        inner.pack_start(&add_topic);

        Self { inner, add_topic }
    }

    pub fn add_action(&self, action: impl Fn() + 'static) {
        self.add_topic.connect_clicked(move |cb| {
            action();
            // rt.sender().send(UiEvent::SetConnection(id1, id2, state));
        });
    }
}

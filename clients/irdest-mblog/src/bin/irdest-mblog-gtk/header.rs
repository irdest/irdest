use crate::{topic::Topics, topic_creator::TopicCreator};
use async_std::sync::Arc;
use gtk::{
    gio::{Menu, MenuItem},
    glib,
    prelude::*,
    ApplicationWindow, Button, HeaderBar, Label, MenuButton,
};
use irdest_mblog::Lookup;

#[allow(unused)]
pub struct Header {
    pub inner: HeaderBar,
    add_topic: Button,
    menu_button: MenuButton,
    menu: Menu,
}

impl Header {
    pub fn new(parent: ApplicationWindow, lookup: Arc<Lookup>, topics: Topics) -> Header {
        let inner = HeaderBar::new();
        inner.set_show_title_buttons(true);
        inner.set_title_widget(Some(&Label::new(Some("Irdest mblog"))));

        let add_topic = Button::from_icon_name("folder-new-symbolic");
        add_topic.connect_clicked(
            glib::clone!(@weak parent => move |_| TopicCreator::new(parent, Arc::clone(&lookup), topics.clone())),
        );

        let menu_button = MenuButton::new();
        menu_button.set_icon_name("open-menu");

        let menu = Menu::new();
        menu.append_item(&MenuItem::new(Some("About mblog"), Some("app.about")));
        menu.append_item(&MenuItem::new(Some("Quit"), Some("app.quit")));
        menu_button.set_menu_model(Some(&menu));

        inner.pack_start(&add_topic);
        inner.pack_end(&menu_button);

        Self {
            inner,
            add_topic,
            menu_button,
            menu,
        }
    }

    // FIXME: not sure why this function is unused!
    pub fn add_action(&self, action: impl Fn() + 'static) {
        self.add_topic.connect_clicked(move |_| {
            action();
        });
    }
}

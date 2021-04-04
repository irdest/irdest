use crate::widgets::content::Content;

use gtk::{prelude::*, GtkWindowExt};
use relm::Widget;
use relm_derive::{widget, Msg};

pub struct Model {}

#[derive(Debug, Msg)]
pub enum Update {
    Quit,
}

#[widget]
impl Widget for Window {
    view! {
        #[name = "window"]
        gtk::ApplicationWindow {
            title: concat!("qaul GTK (v", env!("CARGO_PKG_VERSION"), ")"),

            Content {},

            delete_event(_, _) => (Update::Quit, Inhibit(false)),
        }
    }

    fn init_view(&mut self) {
        self.widgets.window.set_default_size(800, 600);
    }

    fn update(&mut self, event: Update) {
        match dbg!(event) {
            Update::Quit => gtk::main_quit(),
        }
    }

    fn model() -> Model {
        Model {}
    }
}

pub fn run() {
    Window::run(()).unwrap();
}

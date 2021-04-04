use crate::widgets::screens::Login;

use gtk::{prelude::*, Align, Orientation::*};
use relm::Widget;
use relm_derive::{widget, Msg};

pub struct Model {}

#[derive(Debug, Msg)]
pub enum Update {}

#[widget]
impl Widget for Content {
    fn model() -> Model {
        Model {}
    }

    fn update(&mut self, _: Update) {}

    view! {
        #[name = "content"]
        gtk::Box {
            orientation: Vertical,
            valign: Align::Center,

            Login {}
            
            // gtk::Label {
            //     text: "Nothing to see here",
            // }
        }
    }
}

use super::{Command, Model};

use gtk::{prelude::*, Align::Center, InputPurpose::Password, Orientation::*};
use relm::Widget;
use relm_derive::widget;

#[widget]
impl Widget for Login {
    view! {
        gtk::Box {
            orientation: Vertical,

            #[name = "greeting"]
            gtk::Label {
                text: "Welcome to qaul!",
            },
            gtk::Box {
                orientation: Horizontal,
                halign: Center,

                #[name = "user_list"]
                gtk::ComboBoxText { },
            },
            gtk::Box {
                orientation: Horizontal,
                halign: Center,

                #[name = "password"]
                gtk::Entry {
                    placeholder_text: Some("Your password"),
                    input_purpose: Password,
                },

                gtk::Button {
                    label: "Create",
                    clicked => Command::Login { id: "".into(), password: "".into(), },
                },
            }

        }
    }

    fn init_view(&mut self) {
        self.widgets.greeting.set_margin_bottom(15);

        // TODO: get available users

        self.widgets
            .user_list
            .insert_text(0, "Create a new user ID");
        self.widgets.user_list.set_active(Some(0));
    }

    fn update(&mut self, _event: Command) {}

    fn model() -> Model {
        Model {}
    }
}

use super::{Command, Model};

use gtk::{
    prelude::*,
    Align::{Center, End},
    InputPurpose::Password,
    Orientation::*,
};
use relm::Widget;
use relm_derive::widget;

#[widget]
impl Widget for Login {
    view! {

        gtk::Grid {
            halign: Center,
            row_spacing: 5,
            column_spacing: 15,

            gtk::Label {
                text: "Welcome to qaul!",
                margin_bottom: 15,
                halign: Center,
                cell: { left_attach: 0, top_attach: 0, width: 2 },
            },

            gtk::Label {
                text: "Select user",
                halign: End,
                cell: { left_attach: 0, top_attach: 1}
            },

            #[name = "user_list"]
            gtk::ComboBoxText {
                cell: { left_attach: 1, top_attach: 1 },
            },

            gtk::Label {
                text: "Password",
                halign: End,
                cell: { left_attach: 0, top_attach: 2 }
            },

            #[name = "password"]
            gtk::Entry {
                placeholder_text: Some("*********"),
                input_purpose: Password,
                visibility: false,
                cell: { left_attach: 1, top_attach: 2 },
            },

            gtk::Button {
                label: "Login",
                clicked => Command::Login { id: "".into(), password: "".into(), },
                cell: { left_attach: 0, top_attach: 3, width: 2 },
            }

        }
    }

    fn init_view(&mut self) {
        // TODO: get available users

        dbg!(self.widgets.password.get_input_purpose());
        
        self.widgets.user_list.insert_text(0, "Create new ID");
        self.widgets.user_list.set_active(Some(0));
    }

    fn update(&mut self, _event: Command) {}

    fn model() -> Model {
        Model {}
    }
}

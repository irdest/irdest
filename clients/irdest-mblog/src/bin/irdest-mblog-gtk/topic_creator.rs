use gtk::{
    prelude::*, Align, ApplicationWindow, Box as GtkBox, Dialog, DialogFlags, EditableLabel, Entry,
    InputHints, Justification, Label, Orientation, ResponseType,
};

/// A UI that enables users to create topics that are namespaced along
/// existing categories.
pub struct TopicCreator {
    inner: Dialog,
}

fn create_entry(placeholder: &str) -> Entry {
    let e = Entry::new();
    e.set_placeholder_text(Some(&placeholder));
    e.set_input_hints(InputHints::LOWERCASE);
    e
}

fn create_label(label: &str) -> Label {
    let l = Label::new(Some(label));
    l.set_justify(Justification::Left);
    l.set_halign(Align::Start);
    l
}

impl TopicCreator {
    pub fn new(parent: ApplicationWindow) {
        let inner = Dialog::with_buttons(
            Some("Create a new topic"),
            Some(&parent),
            DialogFlags::empty(),
            &[
                ("Cancel", ResponseType::Cancel),
                ("Create", ResponseType::Apply),
            ],
        );

        let area = inner.content_area();
        area.set_margin_top(64);
        area.set_margin_bottom(64);
        area.set_margin_start(64);
        area.set_margin_end(64);
        area.set_hexpand(true);
        area.set_vexpand(true);
        area.set_spacing(8);
        area.set_orientation(Orientation::Vertical);

        let up_row = GtkBox::new(Orientation::Vertical, 0);

        up_row.append(&create_label("Create a new Topic!"));
        up_row.append(&create_label(
            "A topic consists of a CATEGORY, a NAMESPACE, and a TOPIC NAME.",
        ));
        up_row.append(&create_label("No spaces are allowed!"));

        let down_row = GtkBox::new(Orientation::Horizontal, 0);
        down_row.append(&Label::new(Some("/")));
        down_row.append(&create_entry("<Category>"));
        down_row.append(&Label::new(Some("/")));
        down_row.append(&create_entry("<Namespace>"));
        down_row.append(&Label::new(Some("/")));
        down_row.append(&create_entry("<Topic>"));

        area.append(&up_row);
        area.append(&down_row);

        inner.show();
    }
}

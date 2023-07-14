use crate::topic::{Topic, Topics};
use async_std::sync::Arc;
use gtk::{
    glib::{self, Type},
    prelude::*,
    Align, ApplicationWindow, Box as GtkBox, Button, Dialog, DialogFlags, Entry, EntryCompletion,
    InputHints, Justification, Label, ListStore, Orientation,
};
use irdest_mblog::Lookup;

/// A UI that enables users to create topics that are namespaced along
/// existing categories.
#[derive(Clone)]
#[allow(unused)]
pub struct TopicCreator {
    inner: Dialog,
}

struct SaveButtonClosure {
    category_entry: Entry,
    namespace_entry: Entry,
    topic_entry: Entry,
    topics: Topics,
    _self: Dialog,
}

fn parse_entries(ce: &Entry, ne: &Entry, te: &Entry) -> String {
    let category = ce.buffer().text();
    let namespace = ne.buffer().text();
    let topic = te.buffer().text();

    format!("/{}/{}/{}", category, namespace, topic)
}

fn create_entry(placeholder: &str, lookup: Vec<String>) -> Entry {
    let e = Entry::new();
    e.set_placeholder_text(Some(&placeholder));
    e.set_input_hints(InputHints::LOWERCASE);

    let comp_model = EntryCompletion::new();
    comp_model.set_text_column(0);
    comp_model.set_minimum_key_length(1);
    comp_model.set_popup_completion(true);

    let col_type: [Type; 1] = [Type::STRING];
    let store = ListStore::new(&col_type);
    for elem in lookup {
        store.set(&store.append(), &[(0, &elem)])
    }
    comp_model.set_model(Some(&store));
    e.set_completion(Some(&comp_model));

    // Configure the Entry to automatically replace ` ` with `-`
    e.connect_changed(|_self| {
        println!("TODO: prevent users from entering spaces");
    });

    e
}

fn create_label(label: &str) -> Label {
    let l = Label::new(Some(label));
    l.set_justify(Justification::Left);
    l.set_halign(Align::Start);
    l
}

impl TopicCreator {
    pub fn new(parent: ApplicationWindow, lookup: Arc<Lookup>, topics: Topics) {
        let inner = Dialog::with_buttons(
            Some("Create a new topic"),
            Some(&parent),
            DialogFlags::empty(),
            &[], // ("Cancel", ResponseType::Cancel),
                 // ("Create", ResponseType::Apply),
        );

        let area = inner.content_area();
        area.set_margin_top(32);
        area.set_margin_bottom(32);
        area.set_margin_start(32);
        area.set_margin_end(32);
        area.set_hexpand(true);
        area.set_vexpand(true);
        area.set_spacing(8);
        area.set_orientation(Orientation::Vertical);

        let up_row = GtkBox::new(Orientation::Vertical, 0);
        up_row.append(&create_label(
            "A topic consists of a CATEGORY, a NAMESPACE, and a TOPIC NAME.",
        ));
        up_row.append(&create_label("No spaces are allowed in any segment!"));
        up_row.append(&create_label("Use `-` or `_` instead."));
        up_row.set_margin_bottom(16);

        let down_row = GtkBox::new(Orientation::Horizontal, 0);
        down_row.append(&Label::new(Some("/")));

        let category_entry = create_entry("<Category>", lookup.categories());
        down_row.append(&category_entry);

        down_row.append(&Label::new(Some("/")));

        let namespace_entry = create_entry("<Namespace>", lookup.namespaces());
        down_row.append(&namespace_entry);

        down_row.append(&Label::new(Some("/")));

        // TODO: can we dynamically adjust the input completion based
        // on the namespace a user has already typed?
        let topic_entry = create_entry("<Topic>", vec![]);
        down_row.append(&topic_entry);
        down_row.set_margin_bottom(16);

        let button_row = GtkBox::new(Orientation::Horizontal, 0);
        button_row.set_halign(Align::End);
        button_row.set_hexpand(false);
        button_row.set_spacing(8);

        let btn_cancel = Button::with_label("Cancel");
        btn_cancel.connect_clicked(glib::clone!(@weak inner => move |_| inner.close()));
        button_row.append(&btn_cancel);

        let button_closure = Arc::new(SaveButtonClosure {
            category_entry,
            namespace_entry,
            topic_entry,
            topics,
            _self: inner.clone(),
        });

        let btn_save = Button::with_label("Save");
        button_row.append(&btn_save);
        btn_save.connect_clicked(move |_| {
            let SaveButtonClosure {
                category_entry,
                namespace_entry,
                topic_entry,
                topics,
                _self,
            } = &*Arc::clone(&button_closure);
            let new_topic = parse_entries(&category_entry, &namespace_entry, &topic_entry);
            async_std::task::block_on(topics.add_topic(new_topic.as_str(), Topic::empty()));
            _self.close();
        });

        area.append(&up_row);
        area.append(&down_row);
        area.append(&button_row);

        inner.show();
    }
}

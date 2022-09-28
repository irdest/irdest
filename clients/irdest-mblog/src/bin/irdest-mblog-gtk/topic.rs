use gtk::prelude::*;
use gtk::{
    builders::BoxBuilder, Align, Box as GtkBox, Button, Entry, Label as GtkLabel, Orientation,
    Stack, StackSidebar,
};

/// Topics UI management container
pub struct Topics {
    pub sidebar: StackSidebar,
    pub stack: Stack,
}

impl Topics {
    pub fn new() -> Topics {
        let stack = Stack::new();
        let sidebar = StackSidebar::new();
        sidebar.set_stack(&stack);
        stack.set_vhomogeneous(true);

        Self { stack, sidebar }
    }

    pub fn add_topic(&self, name: &str, child: Topic) {
        self.stack.add_titled(&child.inner, Some(name), name);
    }
}

pub struct Topic {
    inner: GtkBox,
}

impl Topic {
    pub fn empty() -> Self {
        let footer = TopicFooter::new();
        let inner = BoxBuilder::new()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .hexpand(true)
            .vexpand(true)
            .margin_start(32)
            .margin_end(32)
            .halign(Align::Fill)
            .valign(Align::Fill)
            .build();
        Self { inner }
    }

    pub fn add_message(&self, msg: &str) {
        // TODO: create a frame, put the message in there, then add to
        // the inner box
    }
}

/// Display a footer at the bottom of the topic screen
///
/// Renders
pub struct TopicFooter {
    inner: GtkBox,
    entry: Entry,
}

impl TopicFooter {
    pub fn new() -> Self {
        let inner = GtkBox::new(Orientation::Horizontal, 0);
        let entry = Entry::new();
        inner.append(&entry);

        Self { inner, entry }
    }
}

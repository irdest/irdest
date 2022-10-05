use gtk::prelude::*;
use gtk::{
    builders::BoxBuilder, pango::WrapMode, Align, Box as GtkBox, Button, Entry, Frame,
    Label as GtkLabel, Label, NaturalWrapMode, Orientation, PolicyType, ScrolledWindow, Stack,
    StackSidebar,
};
use irdest_mblog::Post;

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
    inner: ScrolledWindow,
    layout: GtkBox,
}

impl Topic {
    pub fn empty() -> Self {
        let footer = TopicFooter::new();
        let layout = BoxBuilder::new()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .hexpand(true)
            .vexpand(true)
            .margin_start(32)
            .margin_end(32)
            .halign(Align::Fill)
            .valign(Align::Fill)
            .build();

        let inner = ScrolledWindow::new();
        inner.set_child(Some(&layout));
        inner.set_hscrollbar_policy(PolicyType::Never);
        inner.set_vscrollbar_policy(PolicyType::Always);

        Self { layout, inner }
    }

    pub fn add_message(&self, msg: &Post) {
        let frame = Frame::new(None);
        frame.set_label(Some(msg.nick.as_str()));

        let child = GtkBox::new(Orientation::Vertical, 0);
        child.set_margin_start(16);
        child.set_margin_end(16);
        child.set_margin_top(8);
        child.set_margin_bottom(8);

        let text = Label::new(Some(msg.text.as_str()));
        text.set_single_line_mode(false);
        text.set_natural_wrap_mode(NaturalWrapMode::Word);
        text.set_wrap_mode(WrapMode::WordChar);
        text.set_selectable(true);
        text.set_wrap(true);
        text.set_halign(Align::Start);

        child.append(&text);
        frame.set_child(Some(&child));
        self.layout.append(&frame);
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

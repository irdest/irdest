use async_std::sync::{Arc, Mutex};
use gtk::prelude::*;
use gtk::{
    builders::BoxBuilder, pango::WrapMode, Align, Box as GtkBox, Entry, Frame, Label,
    NaturalWrapMode, Orientation, PolicyType, ScrolledWindow, Stack, StackSidebar, Viewport,
};
use irdest_mblog::Post;
use std::collections::{BTreeMap, BTreeSet};

use crate::state::AppState;

/// Topics UI management container
#[derive(Clone)]
pub struct Topics {
    pub sidebar: StackSidebar,
    pub stack: Stack,
    data: Arc<Mutex<BTreeMap<String, Topic>>>,
}

impl Topics {
    pub fn new() -> Topics {
        let stack = Stack::new();
        let sidebar = StackSidebar::new();
        let data = Arc::new(Mutex::new(BTreeMap::new()));
        sidebar.set_stack(&stack);
        stack.set_vhomogeneous(true);

        Self {
            stack,
            sidebar,
            data,
        }
    }

    pub fn current_topic(&self) -> String {
        self.stack.visible_child_name().unwrap().to_string()
    }

    pub async fn redraw(&self, topic: &String, state: &Arc<AppState>) {
        let data = self.data.lock().await;
        let t = data.get(topic).unwrap();
        t.clear();

        for msg in state.iter_topic(topic).unwrap() {
            if msg.is_err() {
                continue;
            }

            t.add_message(msg.unwrap().as_post());
        }
    }

    pub async fn setup_notifier(&self, state: Arc<AppState>) {
        let previous_topics: BTreeSet<_> = state.topics().into_iter().collect();

        while let Some(_) = state.wait_topics().await {
            // When this loop runs we need to query topics from the
            // state database and add any new topics (topics can't be
            // deleted/ hidden for now)

            let new_topics: BTreeSet<_> = state.topics().into_iter().collect();
            let diff_topics = new_topics.difference(&previous_topics);

            for topic in diff_topics {
                println!("Adding new topic: '{}'", topic);
                self.add_topic(topic.as_str(), Topic::empty()).await;
            }
        }
    }

    pub async fn add_topic(&self, name: &str, child: Topic) {
        self.stack.add_titled(&child.inner, Some(name), name);
        self.data.lock().await.insert(name.into(), child);
    }
}

pub struct Topic {
    inner: ScrolledWindow,
}

impl Topic {
    pub fn empty() -> Self {
        let _footer = TopicFooter::new();
        let inner = ScrolledWindow::new();
        let this = Self { inner };
        this.clear();
        this
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
        // text.set_selectable(true); // this causes some issues where it auto-selects the first message
        text.set_wrap(true);
        text.set_halign(Align::Start);

        child.append(&text);
        frame.set_child(Some(&child));

        let inner_child = self.inner.child().unwrap();
        let inner_viewport = inner_child.downcast_ref::<Viewport>().unwrap();
        let viewport_child = inner_viewport.child().unwrap();
        let layout_box = viewport_child.downcast_ref::<GtkBox>().unwrap();
        layout_box.append(&frame);
    }

    pub fn clear(&self) {
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

        self.inner.set_child(Some(&layout));
        self.inner.set_hscrollbar_policy(PolicyType::Never);
        self.inner.set_vscrollbar_policy(PolicyType::Always);
    }
}

/// Display a footer at the bottom of the topic screen
///
/// Renders
#[allow(unused)]
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

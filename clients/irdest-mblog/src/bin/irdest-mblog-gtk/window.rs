use crate::{
    footer::Footer,
    header::Header,
    topic::{Topic, Topics},
};
use async_std::sync::Arc;
use gtk::prelude::*;
use gtk::{
    builders::BoxBuilder, Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar,
    Label as GtkLabel, Orientation, Stack, StackSidebar, Statusbar, Window,
};
use irdest_mblog::{Lookup, Post};

pub struct MBlogWindow {
    inner: ApplicationWindow,
    topics: Topics,
    header: Header,
    footer: Footer,
}

impl MBlogWindow {
    pub fn new(app: &Application) -> Self {
        let inner = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Irdest mblog")
            .build();

        // Just hard-code a list of topics for now
        let lookup = Arc::new(Lookup::populate(vec![
            "/net/irdest/general",
            "/net/irdest/bugs",
            "/net/irdest/off-topic",
            "/comp/nixos/general",
            "/sci/radio/general",
            "/local/berlin/rave",
            "/local/berlin/afra",
        ]));

        let topics = Topics::new();
        let header = Header::new(inner.clone(), Arc::clone(&lookup));
        inner.set_titlebar(Some(&header.inner));

        let container = GtkBox::new(Orientation::Vertical, 0);

        let status_bar = Statusbar::new();
        status_bar.push(0, "Establishing connection to Ratman daemon...");

        // the main layout is a box with two segments
        let layout = GtkBox::new(Orientation::Horizontal, 0);
        layout.append(&topics.sidebar);

        let footer = Footer::new();

        // This layout appends a footer under the topic stack so we
        // can re-use the same message footer for all topics.
        let topic_footer_layout = GtkBox::new(Orientation::Vertical, 0);
        topic_footer_layout.set_vexpand(true);
        topic_footer_layout.append(&topics.stack);
        topic_footer_layout.append(&footer.inner);
        layout.append(&topic_footer_layout);

        container.append(&layout);
        container.append(&status_bar);

        // Add the layout to the window
        inner.set_child(Some(&container));

        // Add all known topics to the list
        for topic in lookup.all() {
            let t = Topic::empty();
            t.add_message(&Post {
                nick: "Alice".into(),
                text: "Is this thing on??".into(),
                topic: "".into(),
            });

            t.add_message(&Post {
                nick: "Bob".into(),
                text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Arcu vitae elementum curabitur vitae nunc sed velit. Vel fringilla est ullamcorper eget nulla facilisi etiam. Hac habitasse platea dictumst quisque sagittis purus sit amet volutpat. Suscipit adipiscing bibendum est ultricies integer quis. Quam lacus suspendisse faucibus interdum posuere. Amet nulla facilisi morbi tempus iaculis. Laoreet non curabitur gravida arcu ac. Massa id neque aliquam vestibulum morbi blandit cursus risus. Eu volutpat odio facilisis mauris sit amet massa vitae tortor. Senectus et netus et malesuada fames. Amet nisl suscipit adipiscing bibendum. Amet volutpat consequat mauris nunc congue nisi vitae. Mauris nunc congue nisi vitae suscipit tellus mauris a. Sem et tortor consequat id porta nibh venenatis cras sed. Nisi vitae suscipit tellus mauris a diam maecenas sed. Dui ut ornare lectus sit amet est placerat in.".into(),
                topic: "".into(),
            });

            t.add_message(&Post {
                nick: "Alice".into(),
                text: "Dude okay we get it, you studied Latin...".into(),
                topic: "".into(),
            });

            topics.add_topic(topic.as_str(), t);
        }

        Self {
            inner,
            topics,
            header,
            footer,
        }
    }

    pub fn show(&self) {
        self.inner.show();
        self.inner.present();
    }
}

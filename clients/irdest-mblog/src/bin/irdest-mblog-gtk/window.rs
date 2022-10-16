use crate::{
    footer::Footer,
    header::Header,
    state::AppState,
    topic::{Topic, Topics},
};
use async_std::sync::Arc;
use gtk::prelude::*;
use gtk::{
    builders::BoxBuilder, glib, Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar,
    Label as GtkLabel, Orientation, Stack, StackSidebar, Statusbar, Window,
};
use irdest_mblog::{Lookup, Message, Payload, Post};

pub struct MBlogWindow {
    inner: ApplicationWindow,
    topics: Topics,
    header: Header,
    footer: Footer,
}

impl MBlogWindow {
    pub fn new(app: &Application, state: Arc<AppState>) -> Self {
        let inner = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Irdest mblog")
            .build();

        let mut tops = state.topics();
        tops.push("/net/irdest/welcome".into());
        let lookup = Arc::new(Lookup::populate(tops));

        // Just hard-code a list of topics for now
        // let lookup = Arc::new(Lookup::populate(vec![
        //     "/net/irdest/general",
        //     "/net/irdest/bugs",
        //     "/net/irdest/off-topic",
        //     "/comp/nixos/general",
        //     "/sci/radio/general",
        //     "/local/berlin/rave",
        //     "/local/berlin/afra",
        // ]));

        // Create a topics container and populate a starting topic from the database
        let topics = Topics::new();

        topics.add_topic(
            "/net/irdest/welcome",
            state
                .iter_topic("/net/irdest/welcome")
                .expect("failed to load post database!")
                .fold(Topic::empty(), |mut t, msg| {
                    match msg {
                        Ok(Message {
                            payload: Payload::Post(ref p),
                            ..
                        }) => t.add_message(p),
                        _ => {}
                    }
                    t
                }),
        );

        {
            // Setup a task that adds topics as they are discovered
            let topics = topics.clone();
            let state = Arc::clone(&state);

            glib::MainContext::default().spawn_local(async move {
                topics.setup_notifier(state).await;
            });
        }

        {
            // Setup a task that adds messages to a topic if it selected
            let topics = topics.clone();
            let state = Arc::clone(&state);

            glib::MainContext::default().spawn_local(async move {
                while let Some(redraw_topic) = state.wait_dirty().await {
                    let t = topics.get_topic(redraw_topic.as_str()).unwrap();
                    t.clear();
                    for msg in state.iter_topic(redraw_topic) {
                        t.add_message(msg);
                    }
                }
            });
        }

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
        // for topic in lookup.all() {
        //     let t = Topic::empty();
        //     t.add_message(&Post {
        //         nick: "Alice".into(),
        //         text: "Is this thing on??".into(),
        //         topic: "".into(),
        //     });

        //     t.add_message(&Post {
        //         nick: "Bob".into(),
        //         text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Arcu vitae elementum curabitur vitae nunc sed velit. Vel fringilla est ullamcorper eget nulla facilisi etiam. Hac habitasse platea dictumst quisque sagittis purus sit amet volutpat. Suscipit adipiscing bibendum est ultricies integer quis. Quam lacus suspendisse faucibus interdum posuere. Amet nulla facilisi morbi tempus iaculis. Laoreet non curabitur gravida arcu ac. Massa id neque aliquam vestibulum morbi blandit cursus risus. Eu volutpat odio facilisis mauris sit amet massa vitae tortor. Senectus et netus et malesuada fames. Amet nisl suscipit adipiscing bibendum. Amet volutpat consequat mauris nunc congue nisi vitae. Mauris nunc congue nisi vitae suscipit tellus mauris a. Sem et tortor consequat id porta nibh venenatis cras sed. Nisi vitae suscipit tellus mauris a diam maecenas sed. Dui ut ornare lectus sit amet est placerat in.".into(),
        //         topic: "".into(),
        //     });

        //     t.add_message(&Post {
        //         nick: "Alice".into(),
        //         text: "Dude okay we get it, you studied Latin...".into(),
        //         topic: "".into(),
        //     });

        //     topics.add_topic(topic.as_str(), t);
        // }

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

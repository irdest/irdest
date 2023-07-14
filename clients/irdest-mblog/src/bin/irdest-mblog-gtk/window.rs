use crate::{
    footer::Footer,
    header::Header,
    state::AppState,
    topic::{Topic, Topics},
};
use async_std::sync::Arc;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Box as GtkBox, Orientation, Statusbar};
use irdest_mblog::Lookup;

#[allow(unused)]
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

        async_std::task::block_on(async {
            let known_topics = state.topics();
            for topic in known_topics {
                topics.add_topic(&topic, Topic::empty()).await;
                topics.redraw(&topic, &state).await;
            }
        });

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
                // Get notified when a topic is dirty so we can redraw
                // it (if it's currently in view)
                while let Some(dirty) = state.wait_dirty().await {
                    // TODO: only redraw current topic but for this we
                    // need to also redraw on topic change which we
                    // currently can't because the Gtk signal system
                    // is strange
                    // if topics.current_topic() != dirty {
                    //     continue;
                    // }

                    topics.redraw(&dirty, &state).await;
                }
            });
        }

        let header = Header::new(inner.clone(), Arc::clone(&lookup), topics.clone());
        inner.set_titlebar(Some(&header.inner));

        let container = GtkBox::new(Orientation::Vertical, 0);

        let status_bar = Statusbar::new();
        status_bar.push(0, "Establishing connection to Ratman daemon...");

        // the main layout is a box with two segments
        let layout = GtkBox::new(Orientation::Horizontal, 0);
        layout.append(&topics.sidebar);

        let footer = Footer::new(Arc::clone(&state), topics.clone());

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

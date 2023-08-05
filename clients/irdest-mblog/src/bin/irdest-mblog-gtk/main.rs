use anyhow::{anyhow, Result};
use async_std::sync::Arc;
use gtk::{gio::SimpleAction, glib, prelude::*, Application};
use irdest_mblog::Message;
use libratman::client::RatmanIpc;

mod footer;
mod header;
mod state;
mod topic;
mod topic_creator;
mod window;

#[async_std::main]
async fn main() -> Result<()> {
    // Load the resources we compiled into the binary (build_resourcesi() in build.rs).
    // If this fails, the problem is probably in there or in the XML, not here.
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("failed to register resources; this should never happen");

    let app = Application::builder()
        .application_id("org.irdest.mblog_gtk")
        .build();

    let dirs = directories::ProjectDirs::from("org", "irdest", "irdest-mblog")
        .ok_or(anyhow!("couldn't find config dir"))?;
    let db = sled::open(dirs.data_local_dir().join("db"))?;

    let (new_user, addr, token) = irdest_mblog::load_or_create_addr().await?;
    let ipc = RatmanIpc::default_with_addr(addr, token).await?;
    println!("Running with address: {}", addr);

    let state = Arc::new(crate::state::AppState::new(ipc, db));

    // If the user opened the app for the first time (probably) we
    // create a small welcome message for them
    if new_user {
        if let Err(e) = state.parse_and_store(&Message::generate_intro(addr)) {
            eprintln!("Error occured while creating a new user: {}", e);
        }
    }

    let receiver_state = Arc::clone(&state);
    glib::MainContext::default().spawn_local(async move {
        let mut previous_topics = vec![];

        loop {
            match receiver_state.next().await {
                Ok(None) => {}
                Ok(Some(msg)) => {
                    let new_topics = receiver_state.topics();

                    println!("New message: {:?}", msg);
                    println!("Previous topics: {}", previous_topics.len());
                    println!("New topics: {}", new_topics.len());

                    // If a new topic was added, notify topics to re-draw
                    if new_topics.len() > previous_topics.len() {
                        receiver_state.notify_topics().await;
                    }
                    previous_topics = new_topics;

                    // Afterwards notify the topic that received a new message
                    receiver_state.notify_dirty(&msg.as_post().topic).await;
                }
                Err(e) => eprintln!("input error: {}", e),
            }
        }
    });

    // TODO: replace with gio::ActionEntry::builder in the future
    let action = SimpleAction::new("quit", None);
    action.connect_activate(glib::clone!(@weak app => move |_, _| app.quit()));
    app.add_action(&action);

    app.connect_activate(move |app| {
        let window = window::MBlogWindow::new(app, Arc::clone(&state));
        window.show();
    });

    app.run();
    Ok(())
}

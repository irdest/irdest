use anyhow::{anyhow, Result};
use async_std::stream::StreamExt;
use async_std::sync::Arc;
use gtk::{gio::SimpleAction, glib, prelude::*, Application};

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

    let addr = irdest_mblog::load_or_create_addr().await?;
    let ipc = ratman_client::RatmanIpc::default_with_addr(addr).await?;
    println!("Running with address: {}", addr);

    let state = Arc::new(crate::state::AppState::new(ipc, db));
    glib::MainContext::default().spawn_local(async move {
        while let Some(msgr) = state.next().await {
            match msgr {
                Ok(msg) => println!("{:?}", msg),
                Err(e) => eprintln!("stream error: {}", e),
            }
        }
    });

    // TODO: replace with gio::ActionEntry::builder in the future
    let action = SimpleAction::new("quit", None);
    action.connect_activate(glib::clone!(@weak app => move |_, _| app.quit()));
    app.add_action(&action);

    app.connect_activate(|app| {
        let window = window::MBlogWindow::new(app);
        window.show();
    });

    app.run();
    Ok(())
}

use gtk::{gio::SimpleAction, glib, prelude::*, Application};

mod footer;
mod header;
mod post;
mod topic;
mod topic_creator;
mod window;

fn main() {
    // Load the resources we compiled into the binary (build_resourcesi() in build.rs).
    // If this fails, the problem is probably in there or in the XML, not here.
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("failed to register resources; this should never happen");

    let app = Application::builder()
        .application_id("org.irdest.mblog_gtk")
        .build();

    // TODO: replace with gio::ActionEntry::builder in the future
    let action = SimpleAction::new("quit", None);
    action.connect_activate(glib::clone!(@weak app => move |_, _| app.quit()));
    app.add_action(&action);

    app.connect_activate(|app| {
        let window = window::MBlogWindow::new(app);
        window.show();
    });

    app.run();
}

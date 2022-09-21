use gtk::prelude::*;
use gtk::Application;

mod window;

fn main() {
    // Load the resources we compiled into the binary (build_resourcesi() in build.rs).
    // If this fails, the problem is probably in there or in the XML, not here.
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("failed to register resources; this should never happen");

    let app = Application::builder()
        .application_id("org.irdest.irdest-mblog")
        .build();

    app.connect_activate(|app| {
        let window = window::Window::new(app);
        window.present();
    });

    app.run();
}

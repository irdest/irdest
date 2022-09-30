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
use irdest_mblog::Lookup;

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

// use glib::Object;
// use gtk::{gio, glib, Application};

// glib::wrapper! {
//     pub struct Window(ObjectSubclass<imp::Window>)
//         @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
//         @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
//                     gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
// }

// impl Window {
//     pub fn new(app: &Application) -> Self {
//         Object::new(&[("application", app)]).expect("Failed to create Window")
//     }
// }

// mod imp {
//     use glib::subclass::InitializingObject;
//     use gtk::prelude::*;
//     use gtk::subclass::prelude::*;
//     use gtk::{glib, CompositeTemplate};

//     // Object holding the state
//     #[derive(CompositeTemplate, Default)]
//     #[template(resource = "/org/irdest/irdest-mblog-gtk/window.ui")]
//     pub struct Window {}

//     // The central trait for subclassing a GObject
//     #[glib::object_subclass]
//     impl ObjectSubclass for Window {
//         const NAME: &'static str = "IrdestMblogMainWindow";
//         type Type = super::Window;
//         type ParentType = gtk::ApplicationWindow;

//         fn class_init(klass: &mut Self::Class) {
//             klass.bind_template();
//             klass.bind_template_callbacks();
//         }

//         fn instance_init(obj: &InitializingObject<Self>) {
//             obj.init_template();
//         }
//     }

//     #[gtk::template_callbacks]
//     impl Window {
//         #[template_callback]
//         fn handle_button_clicked(button: &gtk::Button) {
//             // Set the label to "Hello World!" after the button has been clicked on
//             button.set_label("Hello World!");
//         }
//     }

//     // Trait shared by all GObjects
//     impl ObjectImpl for Window {}

//     // Trait shared by all widgets
//     impl WidgetImpl for Window {}

//     // Trait shared by all windows
//     impl WindowImpl for Window {}

//     // Trait shared by all application windows
//     impl ApplicationWindowImpl for Window {}
// }

use crate::topic::{Topic, Topics};
use gtk::prelude::*;
use gtk::{
    builders::BoxBuilder, Application, ApplicationWindow, Box as GtkBox, Button, Label as GtkLabel,
    Orientation, Stack, StackSidebar, Window,
};

pub struct MBlogWindow {
    inner: ApplicationWindow,
    topics: Topics,
}

impl MBlogWindow {
    pub fn new(app: &Application) -> Self {
        let inner = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Irdest mblog")
            .build();

        let topics = Topics::new();

        // The main layout is a box with two segments
        let layout = GtkBox::new(Orientation::Horizontal, 0);
        layout.append(&topics.sidebar);
        layout.append(&topics.stack);

        // Add the layout to the window
        inner.set_child(Some(&layout));

        // Create topic A
        let topic_a = Topic::new(|layout| {
            let label = GtkLabel::new(Some("Here be networking talk"));
            layout.append(&label);
        });
        topics.add_topic("networking.irdest.general", topic_a);

        // Create topic B
        let topic_b = Topic::new(|layout| {
            let label = GtkLabel::new(Some("Here be bug talk"));
            layout.append(&label);
        });
        topics.add_topic("networking.irdest.bugs", topic_b);

        Self { inner, topics }
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

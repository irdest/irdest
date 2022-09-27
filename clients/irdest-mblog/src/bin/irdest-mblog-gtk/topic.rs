use gtk::{Box as GtkBox, Button, Label as GtkLabel, Orientation, Stack, StackSidebar};

/// Topics UI management container
pub struct Topics {
    pub sidebar: StackSidebar,
    pub stack: Stack,
}

impl Topics {
    pub fn new() -> Topics {
        let stack = Stack::new();
        let sidebar = StackSidebar::new();
        sidebar.set_stack(&stack);

        Self { stack, sidebar }
    }

    pub fn add_topic(&self, name: &str, child: Topic) {
        self.stack.add_titled(&child.inner, Some(name), name);
    }
}

pub struct Topic {
    inner: GtkBox,
}

impl Topic {
    pub fn new(initialise: impl Fn(&GtkBox)) -> Self {
        let inner = GtkBox::new(Orientation::Vertical, 0);
        initialise(&inner);
        Self { inner }
    }
}

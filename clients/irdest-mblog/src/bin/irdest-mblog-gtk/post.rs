use glib::Object;
use gtk::{glib, prelude::*};
use irdest_mblog::Post;

glib::wrapper! {
    /// GObject wrapper for a Post.
    pub struct PostObject(ObjectSubclass<imp::PostObject>);
}

impl From<Post> for PostObject {
    fn from(p: Post) -> Self {
        // TODO: Can we like, not re-construct the inner Post from props here?
        Object::new(&[("nick", &p.nick), ("text", &p.text), ("topic", &p.topic)])
            .expect("Failed to convert `Post` into `PostObject`.")
    }
}

impl From<PostObject> for Post {
    fn from(obj: PostObject) -> Self {
        // TODO: Same here, can we just use the Post we already have instead?
        Self {
            nick: obj.property::<String>("nick"),
            text: obj.property::<String>("text"),
            topic: obj.property::<String>("topic"),
        }
    }
}

mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;

    use glib::{ParamSpec, ParamSpecString, Value};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use irdest_mblog::Post;

    // Object holding the state
    #[derive(Default)]
    pub struct PostObject {
        pub data: Rc<RefCell<Post>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PostObject {
        const NAME: &'static str = "MBlogPostObject";
        type Type = super::PostObject;
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PostObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("nick").build(),
                    ParamSpecString::builder("text").build(),
                    ParamSpecString::builder("topic").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "nick" => {
                    self.data.borrow_mut().nick = value
                        .get()
                        .expect("The value needs to be of type `String`.");
                }
                "text" => {
                    self.data.borrow_mut().text = value
                        .get()
                        .expect("The value needs to be of type `String`.");
                }
                "topic" => {
                    self.data.borrow_mut().topic = value
                        .get()
                        .expect("The value needs to be of type `String`.");
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "nick" => self.data.borrow().nick.to_value(),
                "text" => self.data.borrow().text.to_value(),
                "topic" => self.data.borrow().topic.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PostObject;

    use gtk::prelude::*;
    use irdest_mblog::Post;

    #[test]
    fn test_from_into_post() {
        let post = Post {
            nick: "mike".into(),
            text: "hello, joe".into(),
            topic: "erlang".into(),
        };

        let obj: PostObject = post.clone().into();
        assert_eq!("mike".to_string(), obj.property::<String>("nick"));
        assert_eq!("hello, joe".to_string(), obj.property::<String>("text"));
        assert_eq!("erlang".to_string(), obj.property::<String>("topic"));

        assert_eq!(post, obj.into());
    }
}

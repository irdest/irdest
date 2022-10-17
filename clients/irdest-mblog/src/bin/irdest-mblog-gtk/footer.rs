use crate::{state::AppState, topic::Topics};
use async_std::sync::Arc;
use gtk::{
    builders::BoxBuilder, glib, prelude::*, Box as GtkBox, Button, Entry, Orientation, TextBuffer,
    TextView,
};
use irdest_mblog::{Message, Post, NAMESPACE};
use protobuf::Message as _;

pub struct Footer {
    pub inner: GtkBox,
}

impl Footer {
    pub fn new(state: Arc<AppState>, topics: Topics) -> Self {
        let inner = BoxBuilder::new()
            .orientation(Orientation::Horizontal)
            .margin_start(4)
            .margin_end(4)
            .hexpand(true)
            .build();
        let entry = TextView::new();
        entry.set_hexpand(true);
        entry.set_monospace(true);

        let send = Button::from_icon_name("media-record-symbolic");

        {
            let entry = entry.clone();
            send.connect_clicked(move |_| {
                let state = Arc::clone(&state);
                let buffer = entry.buffer();
                let (mut start, mut end) = buffer.bounds();
                let text = buffer.text(&start, &end, false);
                buffer.delete(&mut start, &mut end);
                let topic = topics.current_topic();

                async_std::task::spawn(async move {
                    let ipc = state.ipc.as_ref().unwrap();
                    let msg = Message::new(Post {
                        nick: ipc.address().to_string(),
                        text: text.to_string(),
                        topic,
                    });

                    println!("Sending message: {:?}", msg);
                    ipc.flood(
                        NAMESPACE.into(),
                        msg.into_proto().write_to_bytes().unwrap(),
                        true, // we want Ratman to send the message back to us
                    )
                    .await
                    .unwrap();
                });
            });
        }

        inner.append(&entry);
        inner.append(&send);

        Self { inner }
    }
}

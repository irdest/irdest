use anyhow::Result;
use async_std::{
    channel::{bounded, Receiver, Sender},
    sync::{Arc, RwLock},
};
use irdest_mblog::{Envelope, Message, Payload, NAMESPACE};
use libratman::{
    client::RatmanIpc,
    types::{Message as RatmanMessage, ApiRecipient},
};
use protobuf::Message as _;
use std::convert::{TryFrom, TryInto};

/// Central app state type which handles connection to Ratman
#[allow(unused)]
pub struct AppState {
    pub ipc: Option<RatmanIpc>,
    db: sled::Db,
    topics_notify: (Sender<()>, Receiver<()>),
    dirty_notify: (Sender<String>, Receiver<String>),
    current_topic: RwLock<Option<String>>,
}

impl AppState {
    // This function should be used to allow irdest-mblog to start,
    // load existing messages, but not rely on a daemon connection.
    #[allow(unused)]
    pub fn new_offline(db: sled::Db) -> Self {
        let topics_notify = bounded(2);
        let dirty_notify = bounded(2);
        Self {
            ipc: None,
            db,
            topics_notify,
            dirty_notify,
            current_topic: RwLock::new(None),
        }
    }

    pub fn new(ipc: RatmanIpc, db: sled::Db) -> Self {
        let topics_notify = bounded(2);
        let dirty_notify = bounded(2);
        Self {
            ipc: Some(ipc),
            db,
            topics_notify,
            dirty_notify,
            current_topic: RwLock::new(None),
        }
    }

    /// Notify the topics poller that a new topic was discovered
    pub async fn notify_topics(&self) {
        if let Err(e) = self.topics_notify.0.send(()).await {
            eprintln!("error occured while notifying topics: {}", e);
        }
    }

    pub async fn wait_topics(self: &Arc<Self>) -> Option<()> {
        self.topics_notify.1.recv().await.ok()
    }

    /// Notify the re-draw poller that `t` is a dirty dirty topic uwu
    pub async fn notify_dirty(&self, t: &String) {
        if let Err(e) = self.dirty_notify.0.send(t.clone()).await {
            eprintln!("error occured while notifying dirty state: {}", e);
        }
    }

    pub async fn wait_dirty(&self) -> Option<String> {
        self.dirty_notify.1.recv().await.ok()
    }

    // FIXME: not sure why this function is unused!
    pub async fn set_topic(&self, top: &str) {
        let mut x = self.current_topic.write().await;
        *x = Some(top.into());
    }

    // FIXME: not sure why this function is unused!
    pub async fn current_topic(&self) -> String {
        self.current_topic.read().await.as_ref().unwrap().clone()
    }

    pub async fn next(&self) -> Result<Option<Message>> {
        if let Some((_tt, ratmsg)) = self
            .ipc
            .as_ref()
            .unwrap()
            .next()
            .await
            // Drop flood messages for the wrong namespace.
            .filter(|(_tt, ratmsg)| match ratmsg.get_recipient() {
                ApiRecipient::Flood(ns) => ns == NAMESPACE.into(),
                ApiRecipient::Standard(_) => true,
            })
        {
            self.parse_and_store(&ratmsg)
        } else {
            Ok(None)
        }
    }

    pub fn parse_and_store(&self, ratmsg: &RatmanMessage) -> Result<Option<Message>> {
        // Track seen IDs, drop duplicate messages.
        let seen_ids = self.db.open_tree("seen_ids")?;
        if let Err(_) = seen_ids.compare_and_swap(
            ratmsg.get_id().as_bytes(),
            None::<&[u8]>,
            Some::<&[u8]>(&chrono::Utc::now().timestamp_millis().to_be_bytes()),
        ) {
            return Ok(None);
        }

        // Store new messages in the database.
        let msg = Message::try_from(ratmsg)?;

        // This is an irrefutable pattern because the Payload enum
        // exists specifically for future-proofing the wire format,
        // although it currently only has a single option.
        #[allow(irrefutable_let_patterns)]
        if let Payload::Post(ref post) = msg.payload {
            // Store the raw protobuf message in the database, so that
            // if we receive a message from the ~future~, we don't
            // drop any newfangled fields on the floor.  - ???
            let data = Envelope::from_ratmsg(&ratmsg)
                .into_proto()
                .write_to_bytes()?;

            // 1 topic = 1 database tree.
            let topic_tree = self.db.open_tree(format!("posts_to/{}", post.topic))?;
            let key_time: [u8; 8] = msg.header.time.timestamp_millis().to_be_bytes();
            let key_id: [u8; 32] = msg.header.id.as_bytes().try_into().unwrap();
            let key: Vec<u8> = key_time
                .iter()
                .chain(&key_id)
                .map(|v| *v)
                .collect::<Vec<_>>();
            topic_tree.insert(key, data)?;
        }
        Ok(Some(msg))
    }

    pub fn topics(&self) -> Vec<String> {
        self.db
            .tree_names()
            .iter()
            .filter_map(|key| std::str::from_utf8(key).ok())
            .filter_map(|key| key.strip_prefix("posts_to/"))
            .map(|key| key.into())
            .collect()
    }

    pub fn iter_topic<S: AsRef<str>>(&self, name: S) -> Result<MessageIterator> {
        Ok(MessageIterator::new(
            self.db
                .open_tree(format!("posts_to/{}", name.as_ref()))?
                .iter(),
        ))
    }
}

pub struct MessageIterator {
    inner: sled::Iter,
}

impl MessageIterator {
    fn new(inner: sled::Iter) -> Self {
        Self { inner }
    }

    fn next_(&mut self) -> Result<Option<Message>> {
        if let Some((_, data)) = self.inner.next().transpose()? {
            Ok(Some(Envelope::parse_proto(&data)?.into_message()?))
        } else {
            Ok(None)
        }
    }
}

impl Iterator for MessageIterator {
    type Item = Result<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use irdest_mblog::{Header, Message, Payload, Post, NAMESPACE};
    use libratman::types::{Address, Message as RatmanMessage, ApiRecipient};
    use protobuf::Message as _;

    #[test]
    fn test_store_post() {
        let mut msg = Message {
            header: Header::default(),
            payload: Payload::Post(Post {
                nick: "mike".into(),
                text: "hello, joe".into(),
                topic: "comp/lang/erlang".into(),
            }),
        };
        let ratmsg = RatmanMessage::new(
            Address::random(),
            ApiRecipient::Flood(NAMESPACE.into()),
            msg.clone().into_proto().write_to_bytes().unwrap(),
            vec![],
        );
        msg.header = Header::message(&ratmsg);

        // Open a blank database in a tempdir.
        let tmpdir =
            tempdir::TempDir::new("irdest-mblog-state-test").expect("couldn't create tempdir");
        let state = AppState::new_offline(sled::open(tmpdir).expect("couldn't open db"));

        // Check that an empty database returns empty lists, not errors.
        assert_eq!(state.topics(), Vec::<String>::new());
        assert_eq!(
            state
                .iter_topic("comp/lang/erlang")
                .expect("couldn't iter_topic")
                .map(|v| v.expect("error in iter_topic"))
                .collect::<Vec<Message>>(),
            Vec::<Message>::new()
        );

        // Parse and store the message.
        let msg2 = state
            .parse_and_store(&ratmsg)
            .expect("couldn't parse_and_store ratmsg")
            .expect("no message returned from parse_and_store");
        assert_eq!(msg, msg2);
        assert_eq!(state.topics(), vec!["comp/lang/erlang"]);
        assert_eq!(
            state
                .iter_topic("comp/lang/erlang")
                .expect("couldn't iter_topic")
                .map(|v| v.expect("error in iter_topic"))
                .collect::<Vec<Message>>(),
            vec![msg.clone()],
        );

        // Re-parsing the same message shouldn't insert a duplicate.
        let msg3 = state
            .parse_and_store(&ratmsg)
            .expect("couldn't parse_and_store ratmsg (2)")
            .expect("no message returned from parse_and_store (2)");
        assert_eq!(msg, msg3);
        assert_eq!(state.topics(), vec!["comp/lang/erlang"]);
        assert_eq!(
            state
                .iter_topic("comp/lang/erlang")
                .expect("couldn't iter_topic")
                .map(|v| v.expect("error in iter_topic"))
                .collect::<Vec<Message>>(),
            vec![msg.clone()],
        );
    }
}

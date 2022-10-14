use anyhow::Result;
use irdest_mblog::{Envelope, Message, Payload, NAMESPACE};
use protobuf::Message as _;
use ratman_client::{RatmanIpc, Recipient};
use std::convert::{TryFrom, TryInto};

/// Central app state type which handles connection to Ratman
pub struct AppState {
    ipc: Option<RatmanIpc>,
    db: sled::Db,
}

impl AppState {
    pub fn new_offline(db: sled::Db) -> Self {
        Self { ipc: None, db }
    }

    pub fn new(ipc: ratman_client::RatmanIpc, db: sled::Db) -> Self {
        Self { ipc: Some(ipc), db }
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
                Recipient::Flood(ns) => ns == NAMESPACE.into(),
                Recipient::Standard(_) => true,
            })
        {
            self.parse_and_store(&ratmsg)
        } else {
            Ok(None)
        }
    }

    fn parse_and_store(&self, ratmsg: &ratman_client::Message) -> Result<Option<Message>> {
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
        // This is an irrefutable pattern because the Payload enum exists specifically for
        // future-proofing the wire format, although it currently only has a single option.
        #[allow(irrefutable_let_patterns)]
        if let Payload::Post(ref post) = msg.payload {
            // Store the raw protobuf message in the database, so that if we receive a
            // message from the ~future~, we don't drop any newfangled fields on the floor.
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
    use protobuf::Message as _;
    use ratman_client::{Address, Recipient};

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
        let ratmsg = ratman_client::Message::new(
            Address::random(),
            Recipient::Flood(NAMESPACE.into()),
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

//! Handling message interaction with Alexandria

use super::Conv;
use crate::messages::{Message, SigTrust};
use alexandria::{record::RecordRef, utils::Diff};

const MID: &'static str = "id";
const SENDER: &'static str = "sender";
const ASSOC: &'static str = "associate";
const SIGN: &'static str = "sign";
const PLOAD: &'static str = "payload";

impl From<RecordRef> for Message {
    fn from(rec: RecordRef) -> Self {
        let kv = rec.kv();

        Self {
            id: Conv::id(kv.get(MID).unwrap()),
            sender: Conv::id(kv.get(SENDER).unwrap()),
            associator: Conv::string(kv.get(ASSOC).unwrap()),
            tags: rec.header.tags.clone(),
            sign: Conv::sig_trust(kv.get(SIGN).unwrap()),
            payload: Conv::binvec(kv.get(PLOAD).unwrap()),
        }
    }
}

impl Message {
    /// Generate a set of diffs to insert into alexandria
    pub(crate) fn diff(&self) -> Vec<Diff> {
        vec![
            Diff::map().insert(MID, self.id.as_bytes().to_vec()),
            Diff::map().insert(SENDER, self.sender.as_bytes().to_vec()),
            Diff::map().insert(ASSOC, self.associator.as_str()),
            Diff::map().insert(
                SIGN,
                match self.sign {
                    SigTrust::Trusted => "trusted",
                    SigTrust::Unverified => "unverified",
                    SigTrust::Invalid => "invalid",
                },
            ),
            Diff::map().insert(PLOAD, self.payload.clone()),
        ]
    }
}
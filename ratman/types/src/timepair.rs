use chrono::{DateTime, Utc};
use regex::Regex;
/// Represents the time of sending and receiving this frame
///
/// Because there is no guarantee that the host clock is accurate or
/// being maliciously manipulated, the sending time should not be
/// trusted.  A timestamp that should be used by applications is
/// available via the `local()` function.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimePair {
    sent: DateTime<Utc>,
    recv: Option<DateTime<Utc>>,
}

impl TimePair {
    /// A utility function to create a new sending timestamp,
    pub fn sending() -> Self {
        Self {
            sent: Utc::now(),
            recv: None,
        }
    }

    /// Update the received time in a timestamp locally received
    pub fn receive(&mut self) {
        self.recv = Some(Utc::now());
    }

    /// A test function to strip the recv-time
    #[doc(hidden)]
    pub fn into_sending(self) -> Self {
        Self { recv: None, ..self }
    }

    /// Get the most likely local time
    pub fn local(&self) -> DateTime<Utc> {
        self.recv.unwrap_or(self.sent)
    }

    /// Capture time(Utc::now) from string and convert to TimePair
    pub fn from_string(s: &str) -> TimePair {
        let re = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.[0-9]*Z").unwrap();

        let times = re
            .captures_iter(s)
            .map(|c| c.get(0).map_or("", |s| s.as_str()))
            .map(|t| t.parse::<DateTime<Utc>>().unwrap())
            .collect::<Vec<_>>();

        if times.is_empty() {
            TimePair {
                sent: Utc::now(), // log: `sent` time is not correct.
                recv: None,
            }
        } else {
            TimePair {
                sent: times[0],
                recv: times.get(1).copied(),
            }
        }
    }
}

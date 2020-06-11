extern crate chrono;

use chrono::prelude::*;
use serde::Deserialize;

#[derive(Clone)]
pub struct AkkaSettings {
    pub tree_address: String,
    pub status_address: String,
    pub dead_letters_address: String,
    pub tree_timeout: u64,
    pub status_timeout: u64,
    pub dead_letters_window: u64,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ActorTreeNode {
    pub name: String,
    pub parent: Option<usize>,
    pub id: usize,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct DeadLettersSnapshot {
    #[serde(rename = "deadLetters")]
    pub dead_letters: Vec<Timestamped<DeadLettersMessage>>,
    pub unhandled: Vec<Timestamped<UnhandledMessage>>,
    pub dropped: Vec<Timestamped<DroppedMessage>>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct Timestamped<T> {
    pub value: T,
    pub timestamp: u64,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct DeadLettersMessage {
    pub message: String,
    pub sender: String,
    pub recipient: String,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct DroppedMessage {
    pub message: String,
    pub sender: String,
    pub recipient: String,
    pub reason: String,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct UnhandledMessage {
    pub message: String,
    pub sender: String,
    pub recipient: String,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct DeadLettersWindow {
    #[serde(rename = "withinMillis")]
    pub within_millis: u64,
    #[serde(rename = "deadLetters")]
    pub dead_letters: DeadLettersWindowData,
    pub unhandled: DeadLettersWindowData,
    pub dropped: DeadLettersWindowData,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct DeadLettersWindowData {
    pub count: u32,
    #[serde(rename = "isMinimumEstimate")]
    pub is_min_estimate: bool,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct DeadLettersMetrics {
    pub snapshot: DeadLettersSnapshot,
    pub window: DeadLettersWindow,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct DeadLettersUIMessage {
    pub message: String,
    pub sender: String,
    pub recipient: String,
    pub timestamp: u64,
    pub reason: Option<String>,
}

#[derive(Deserialize)]
pub struct ActorSystemStatus {
    #[serde(rename(deserialize = "actorCount"))]
    pub actor_count: u64,
    pub uptime: u64,
    #[serde(rename(deserialize = "startTime"))]
    pub start_time: u64,
}


impl DeadLettersWindow {
    pub fn max(&self) -> u32 {
        vec![self.dead_letters.count, self.unhandled.count, self.dropped.count].iter().max().map(|x| x.to_owned()).unwrap_or(0)
    }

    pub fn total(&self) -> u32 {
        vec![self.dead_letters.count, self.unhandled.count, self.dropped.count].iter().sum()
    }
}

impl DeadLettersMessage {
    pub fn to_ui(&self, ts: u64) -> DeadLettersUIMessage {
        DeadLettersUIMessage {
            message: self.message.to_owned(),
            sender: self.sender.to_owned(),
            recipient: self.recipient.to_owned(),
            timestamp: ts,
            reason: None,
        }
    }
}

impl UnhandledMessage {
    pub fn to_ui(&self, ts: u64) -> DeadLettersUIMessage {
        DeadLettersUIMessage {
            message: self.message.to_owned(),
            sender: self.sender.to_owned(),
            recipient: self.recipient.to_owned(),
            timestamp: ts,
            reason: None,
        }
    }
}

impl DroppedMessage {
    pub fn to_ui(&self, ts: u64) -> DeadLettersUIMessage {
        DeadLettersUIMessage {
            message: self.message.to_owned(),
            sender: self.sender.to_owned(),
            recipient: self.recipient.to_owned(),
            timestamp: ts,
            reason: Some(self.reason.to_owned()),
        }
    }
}

impl DeadLettersUIMessage {
    pub fn summary(&self) -> String {
        let now = Local::now().timestamp();
        let diff = now - (self.timestamp / 1000) as i64;
        let diff_min = diff / 60;
        let diff_sec = diff;
        let ago =
            if diff <= 0 {
                "just now".to_owned()
            } else if diff_min > 0 {
                format!("{} min ago", diff_min).to_owned()
            } else {
                format!("{} sec ago", diff_sec).to_owned()
            };
        format!("<{}> {}", ago, self.message)
    }

    pub fn readable_timestamp(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp((self.timestamp / 1000) as i64, 0)
    }
}

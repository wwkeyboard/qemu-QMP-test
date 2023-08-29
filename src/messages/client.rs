//! QMP messages that originate from the client
//!
//! Provides types that model the commands you can send from the client.

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Map, Value};

static NEXT_MESSAGE_ID: AtomicUsize = AtomicUsize::new(1);
fn next_id() -> usize {
    NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub execute: String,
    // TODO: I don't like how general this is, I'd prefer
    // if the type system made it impossible to represent
    // incorrect commands.
    pub arguments: Map<String, Value>,

    pub id: usize,
}

impl Message {
    pub fn new(command: String, arguments: Map<String, Value>) -> Message {
        // TODO: I don't like how this leaks serde types into the caller

        Message {
            id: next_id(),
            execute: command,
            arguments,
        }
    }

    pub fn encode(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

pub fn capabilities() -> Message {
    let mut args = Map::new();
    args.insert("enable".into(), Value::Array(vec!["oob".into()]));

    Message::new("qmp_capabilities".into(), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_capabilities() {
        let result: Message =
            serde_json::from_str(&serde_json::to_string(&capabilities()).unwrap()).unwrap();

        assert_eq!(
            result.arguments.get("enable"),
            Some(&Value::Array(vec![Value::String("oob".into())]))
        );
    }

    #[test]
    fn sets_an_id() {
        // The IDs should start at 1
        let result: Message =
            serde_json::from_str(&serde_json::to_string(&capabilities()).unwrap()).unwrap();
        assert_eq!(result.id, 1)
    }
}

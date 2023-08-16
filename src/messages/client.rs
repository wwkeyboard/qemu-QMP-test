//! QMP messages that originate from the client
//!
//! Provides types that model the commands you can send from the client.

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub execute: String,
    // TODO: I don't like how general this is, I'd prefer
    // if the type system made it impossible to represent
    // incorrect values.
    pub arguments: Map<String, Value>,

    id: usize,
}

impl Message {
    pub fn encode(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

pub fn capabilities(id: usize) -> Message {
    let mut args = Map::new();
    args.insert("enable".into(), Value::Array(vec!["oob".into()]));

    Message {
        execute: "qmp_capabilities".into(),
        arguments: args,
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_capabilities() {
        let result: Message =
            serde_json::from_str(&serde_json::to_string(&capabilities(1)).unwrap()).unwrap();

        assert_eq!(
            result.arguments.get("enable"),
            Some(&Value::Array(vec![Value::String("oob".into())]))
        );
    }

    #[test]
    fn sets_an_id() {
        let id = 1;

        let result: Message =
            serde_json::from_str(&serde_json::to_string(&capabilities(id)).unwrap()).unwrap();
        assert_eq!(result.id, id)
    }
}

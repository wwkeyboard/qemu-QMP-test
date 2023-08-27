//! QMP messages that originate from the server

use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ReceivedMessage {
    Greeting(Box<Greeting>),
    Return(Box<Return>),
}

//
// Greeting message
//

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Greeting {
    #[serde(rename = "QMP")]
    pub qmp: Qmp,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Qmp {
    pub version: Version,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub qemu: Qemu,
    pub package: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Qemu {
    pub micro: u64,
    pub minor: u64,
    pub major: u64,
}

//
// Return message
//

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Return {
    #[serde(rename = "return")]
    pub ret: Value,
    pub id: Option<usize>,
}

pub fn parse(data: String) -> Result<ReceivedMessage> {
    serde_json::from_str(&data).map_err(|e| anyhow!("parsing return data {}", e))
}

#[cfg(test)]
mod tests {
    use std::mem;

    use serde_json::json;

    use super::*;

    #[test]
    fn it_parses() {
        let message = r#"{"QMP": {"version": {"qemu": {"micro": 3, "minor": 0, "major": 8}, "package": ""}, "capabilities": ["oob"]}}"#;
        let p: Greeting = serde_json::from_str(message).unwrap();

        assert_eq!(p.qmp.version.qemu.micro, 3);
    }

    #[test]
    fn identifies_received_messages() {
        let message = r#"{"QMP": {"version": {"qemu": {"micro": 3, "minor": 0, "major": 8}, "package": ""}, "capabilities": ["oob"]}}"#;

        // if this doesn't panic we assume the message parsed correctly
        let _message: ReceivedMessage = serde_json::from_str(message).unwrap();
    }

    #[test]
    fn parses_return_values() {
        let message = String::from(r#"{"return": {}}"#);
        if let ReceivedMessage::Return(data) = parse(message).unwrap() {
            assert_eq!(data.ret, json!({}));
        };
    }

    #[test]
    fn return_message_ids_are_optional() {
        let message = String::from(r#"{"return": {}, "id": 3}"#);

        if let ReceivedMessage::Return(ret) = parse(message).unwrap() {
            assert_eq!(ret.id, Some(3));
        } else {
            panic!("parse didn't find a return")
        }
    }

    #[test]
    fn size_of_received_message_is_box() {
        println!("Return : {:?}", mem::size_of::<ReceivedMessage>());

        panic!("done")
    }
}

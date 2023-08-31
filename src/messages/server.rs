//! QMP messages that originate from the server
//!
//! `ReceivedMessage` is a container for all possible messages, it is the type you should
//! match on to be sure you are handleing all possible cases.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ReceivedMessage {
    Greeting(Box<Greeting>),
    Return(Box<Return>),
    Event(Box<Event>),
}

/// Greeting is the initial message sent when a connection opens. To get QEMU to start sending
/// status information we need to respond with the client capabilities. This is automatically
/// handled by qmp_qsl.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Greeting {
    #[serde(rename = "QMP")]
    pub qmp: Qmp,
}

/// Used by `Greeting`
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Qmp {
    pub version: Version,
    pub capabilities: Vec<String>,
}

/// Used by `Greeting`
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub qemu: Qemu,
    pub package: String,
}

/// Used by `Greeting`
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Qemu {
    pub micro: u64,
    pub minor: u64,
    pub major: u64,
}

/// If you provide an ID with some of the requests the QEMU process will respond with a message
/// also tagged with that ID. The
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Return {
    #[serde(rename = "return")]
    pub ret: Value,
    pub id: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub timestamp: Timestamp,
    pub event: String,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timestamp {
    pub seconds: i64,
    pub microseconds: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub offset: i64,
    #[serde(rename = "qom-path")]
    pub qom_path: String,
}

/// Takes a string of json and tries to turn it into a `ReceivedMessage`
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
        assert_eq!(16, mem::size_of::<ReceivedMessage>());
    }

    #[test]
    fn parses_event_message() {
        let message = String::from(
            r#"{"timestamp": {"seconds": 1693429073, "microseconds": 944495},
  "event": "RTC_CHANGE", 
  "data": {"offset": 1, "qom-path": "/machine/unattached/device[7]"}}"#,
        );

        if let ReceivedMessage::Event(e) = parse(message).unwrap() {
            assert_eq!(e.timestamp.microseconds, 944495);
        } else {
            panic!("didn't parse event correctly");
        }
    }
}

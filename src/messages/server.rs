use serde::Deserialize;
use serde::Serialize;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let message = r#"{"QMP": {"version": {"qemu": {"micro": 3, "minor": 0, "major": 8}, "package": ""}, "capabilities": ["oob"]}}"#;
        let p: Greeting = serde_json::from_str(message).unwrap();

        assert_eq!(p.qmp.version.qemu.micro, 3);
    }
}
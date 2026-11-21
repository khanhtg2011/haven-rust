use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum SocketIoPacket {
    Open {
        sid: String,
        ping_interval: u64,
        ping_timeout: u64,
    },
    Ping,
    Pong,
    ConnectAck {
        sid: String,
    },
    Event {
        name: String,
        payload: Value,
        ack_id: Option<u64>,
    },
    Other(String),
}

pub fn parse_packet(raw: &str) -> Option<SocketIoPacket> {
    if raw.is_empty() {
        return None;
    }

    if raw == "2" {
        return Some(SocketIoPacket::Ping);
    }
    if raw == "3" {
        return Some(SocketIoPacket::Pong);
    }

    if raw.starts_with("40") {
        return Some(SocketIoPacket::ConnectAck {
            sid: "haven-session".into(),
        });
    }

    if raw.starts_with("42") {
        let json_part = &raw[2..];
        if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(json_part) {
            if let Some(first) = arr.first() {
                if let Some(event_name) = first.as_str() {
                    let payload = arr.get(1).cloned().unwrap_or(Value::Null);
                    return Some(SocketIoPacket::Event {
                        name: event_name.to_string(),
                        payload,
                        ack_id: None,
                    });
                }
            }
        }
    }

    Some(SocketIoPacket::Other(raw.to_string()))
}

pub fn encode_event(event_name: &str, payload: &Value) -> String {
    let arr = Value::Array(vec![Value::String(event_name.to_string()), payload.clone()]);
    format!("42{}", serde_json::to_string(&arr).unwrap_or_default())
}

pub fn encode_open(sid: &str, ping_interval: u64, ping_timeout: u64) -> String {
    let obj = serde_json::json!({
        "sid": sid,
        "upgrades": [],
        "pingInterval": ping_interval,
        "pingTimeout": ping_timeout,
        "maxPayload": 1000000
    });
    format!("0{}", obj)
}

pub fn encode_connect_ack(sid: &str) -> String {
    let obj = serde_json::json!({ "sid": sid });
    format!("40{}", obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_framing_roundtrip() {
        assert_eq!(parse_packet("2"), Some(SocketIoPacket::Ping));
        assert_eq!(parse_packet("3"), Some(SocketIoPacket::Pong));

        let payload = json!({"channel_code": "general", "content": "hi"});
        let encoded = encode_event("send-message", &payload);
        assert!(encoded.starts_with("42"));

        let parsed = parse_packet(&encoded);
        match parsed {
            Some(SocketIoPacket::Event { name, payload: p, .. }) => {
                assert_eq!(name, "send-message");
                assert_eq!(p["channel_code"], "general");
            }
            _ => panic!("Expected Event packet"),
        }
    }
}

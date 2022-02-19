#[derive(Debug)]
pub struct Message {
    nanos: u128,
    topic: String,
    payload: Vec<u8>,
}

impl Message {
    pub const fn new(nanos: u128, topic: String, payload: Vec<u8>) -> Self {
        Self {
            nanos,
            topic,
            payload,
        }
    }

    pub fn into_line_protocol(self) -> Option<String> {
        let value = String::from_utf8(self.payload)
            .ok()
            .and_then(|s| floatify(&s))?;
        Some(format!(
            "measurement,topic={} value={} {}",
            line_protocol_escape(&self.topic),
            value,
            self.nanos
        ))
    }
}

/// Assume floats of the payload, otherwise return None
fn floatify(payload: &str) -> Option<f64> {
    let payload = payload.trim();
    if let Ok(payload) = payload.parse::<f64>() {
        Some(payload)
    } else {
        let payload = payload.to_lowercase();
        match payload.as_ref() {
            "true" | "on" | "online" => Some(1.0),
            "false" | "off" | "offline" | "" => Some(0.0),
            _ => None,
        }
    }
}

/// Influx Line Protocol Escape
fn line_protocol_escape(s: &str) -> String {
    s.replace(' ', "\\ ").replace(',', "\\,")
}

#[cfg(test)]
fn test_floatify(input: &str, expected: f64) {
    float_eq::assert_float_eq!(floatify(input).unwrap(), expected, abs <= 0.1);
}

#[test]
fn floatify_int() {
    test_floatify("42", 42.0);
}

#[test]
fn floatify_float() {
    test_floatify("13.37", 13.37);
}

#[test]
fn floatify_bool() {
    test_floatify("true", 1.0);
    test_floatify("false", 0.0);
}

#[test]
fn floatify_on() {
    test_floatify("on", 1.0);
    test_floatify("off", 0.0);
}

#[test]
fn floatify_empty() {
    test_floatify("", 0.0);
    test_floatify("  ", 0.0);
}

#[test]
fn floatify_string() {
    assert!(floatify("whatever").is_none());
}

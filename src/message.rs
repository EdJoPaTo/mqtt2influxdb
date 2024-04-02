use std::fmt::Write;

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

    /// Output as [Line Protocol](https://docs.influxdata.com/influxdb/v2.1/reference/syntax/line-protocol/)
    pub fn into_line_protocol(self) -> Option<String> {
        let value = String::from_utf8(self.payload)
            .ok()
            .and_then(|payload| floatify(&payload))?;
        Some(format!(
            "measurement,{} value={} {}",
            topic_tags(&self.topic),
            value,
            self.nanos
        ))
    }
}

#[test]
fn e2e() {
    let message = Message::new(1337, "foo/bar".into(), b"42".to_vec());
    assert_eq!(
        message.into_line_protocol().unwrap(),
        "measurement,topic=foo/bar,topic1=foo,topic2=bar,topicE1=bar,topic-1=bar,topicE2=foo,topic-2=foo,topicSegments=2 value=42 1337",
    );
}

/// Assume floats of the payload, otherwise return None
fn floatify(payload: &str) -> Option<f64> {
    match payload {
        "true" | "True" | "TRUE" | "on" | "On" | "ON" | "online" | "Online" | "ONLINE" => Some(1.0),
        "false" | "False" | "FALSE" | "off" | "Off" | "OFF" | "offline" | "Offline" | "OFFLINE" => {
            Some(0.0)
        }
        payload => payload
            .split(char::is_whitespace)
            .find(|part| !part.is_empty())? // lazy trim
            .parse::<f64>()
            .ok()
            .filter(|float| float.is_finite()),
    }
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
fn floatify_units() {
    test_floatify("12.3 °C", 12.3);
    test_floatify(" 12.3 °C", 12.3);
}

#[test]
fn floatify_empty() {
    assert!(floatify("").is_none());
    assert!(floatify("  ").is_none());
}

#[test]
fn floatify_string() {
    assert!(floatify("whatever").is_none());
}

#[test]
fn floatify_non_finite() {
    assert!(floatify("nan").is_none());
    assert!(floatify("NaN").is_none());
    assert!(floatify("inf").is_none());
    assert!(floatify("infinity").is_none());
}

/// Influx Line Protocol Escape
fn line_protocol_escape(str: &str) -> String {
    str.replace(' ', "\\ ").replace(',', "\\,")
}

fn topic_tags(topic: &str) -> String {
    let topic = line_protocol_escape(topic);
    let splitted = topic.split('/').collect::<Vec<_>>();
    let mut tags = format!("topic={topic},");
    for (i, part) in splitted.iter().enumerate() {
        _ = write!(&mut tags, "topic{}={part},", i + 1);
    }
    for (i, part) in splitted.iter().rev().take(3).enumerate() {
        _ = write!(&mut tags, "topicE{}={part},", i + 1);
        _ = write!(&mut tags, "topic-{}={part},", i + 1);
    }
    _ = write!(&mut tags, "topicSegments={}", splitted.len());
    tags
}

#[test]
fn topic_tags_short_works() {
    assert_eq!(
        topic_tags("foo/bar"),
        "topic=foo/bar,topic1=foo,topic2=bar,topicE1=bar,topic-1=bar,topicE2=foo,topic-2=foo,topicSegments=2",
    );
}

#[test]
fn topic_tags_long_works() {
    assert_eq!(
        topic_tags("base/foo/bar/test"),
        "topic=base/foo/bar/test,topic1=base,topic2=foo,topic3=bar,topic4=test,topicE1=test,topic-1=test,topicE2=bar,topic-2=bar,topicE3=foo,topic-3=foo,topicSegments=4",
    );
}

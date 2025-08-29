use std::fmt::Write as _;

use crate::payload::{Payload, Values};

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
    pub fn into_line_protocol(self) -> Vec<String> {
        let Some(payload) = Payload::new(self.payload) else {
            return Vec::new();
        };
        let Some(values) = Values::from(&payload) else {
            return Vec::new();
        };
        let topic_tags = topic_tags(&self.topic);
        let Self { nanos, .. } = self;
        match values {
            Values::Single(value) => {
                vec![format!(
                    "measurement,{topic_tags},keySegments=0 value={value} {nanos}"
                )]
            }
            Values::Many(many) => many
                .into_iter()
                .map(|(keys, value)| {
                    let key_tags = key_tags(&keys);
                    format!("measurement,{topic_tags},{key_tags} value={value} {nanos}")
                })
                .collect(),
        }
    }
}

#[cfg(test)]
#[rstest::rstest]
#[case::plain_number(b"42", &["measurement,topic=foo/bar,topic1=foo,topic2=bar,topicE1=bar,topicE2=foo,topicSegments=2,keySegments=0 value=42 1337"])]
#[case::string(b"whatever", &[])]
fn e2e(#[case] payload: &[u8], #[case] expected: &[&str]) {
    let message = Message::new(1337, "foo/bar".into(), payload.to_vec());
    assert_eq!(message.into_line_protocol(), expected);
}

#[test]
fn e2e_json() {
    let payload = serde_json::to_vec(&serde_json::json!({"a": 42, "b": {"c": 666}})).unwrap();
    let message = Message::new(1337, "foo/bar".into(), payload);
    let expected = [
        "measurement,topic=foo/bar,topic1=foo,topic2=bar,topicE1=bar,topicE2=foo,topicSegments=2,key1=a,keySegments=1 value=42 1337",
        "measurement,topic=foo/bar,topic1=foo,topic2=bar,topicE1=bar,topicE2=foo,topicSegments=2,key1=b,key2=c,keySegments=2 value=666 1337",
    ];
    assert_eq!(message.into_line_protocol(), expected);
}

/// Influx Line Protocol Escape
fn line_protocol_escape(str: &str) -> String {
    str.replace(' ', "\\ ").replace(',', "\\,")
}

fn topic_tags(topic: &str) -> String {
    let topic = line_protocol_escape(topic);
    let parts = topic.split('/').collect::<Vec<_>>();
    let mut tags = format!("topic={topic},");
    for (i, part) in parts.iter().enumerate() {
        _ = write!(&mut tags, "topic{}={part},", i + 1);
    }
    for (i, part) in parts.iter().rev().take(3).enumerate() {
        _ = write!(&mut tags, "topicE{}={part},", i + 1);
    }
    _ = write!(&mut tags, "topicSegments={}", parts.len());
    tags
}

#[test]
fn topic_tags_short_works() {
    assert_eq!(
        topic_tags("foo/bar"),
        "topic=foo/bar,topic1=foo,topic2=bar,topicE1=bar,topicE2=foo,topicSegments=2",
    );
}

#[test]
fn topic_tags_long_works() {
    assert_eq!(
        topic_tags("base/foo/bar/test"),
        "topic=base/foo/bar/test,topic1=base,topic2=foo,topic3=bar,topic4=test,topicE1=test,topicE2=bar,topicE3=foo,topicSegments=4",
    );
}

fn key_tags(keys: &[crate::payload::Key<'_>]) -> String {
    let mut tags = String::new();
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            _ = write!(tags, ",");
        }
        _ = write!(tags, "key{}={key}", index.saturating_add(1));
    }
    assert!(!keys.is_empty()); // start with a ,
    _ = write!(tags, ",keySegments={}", keys.len());
    tags
}

#[test]
fn key_tags_works() {
    use crate::payload::Key;
    let keys = [Key::String("foo"), Key::String("bar"), Key::Int(42)];
    let result = key_tags(&keys);
    assert_eq!(result, "key1=foo,key2=bar,key3=42,keySegments=3");
}

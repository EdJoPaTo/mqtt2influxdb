use std::fmt::Write;

use crate::floatify::floatify;

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
        "measurement,topic=foo/bar,topic1=foo,topic2=bar,topicE1=bar,topicE2=foo,topicSegments=2 value=42 1337",
    );
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

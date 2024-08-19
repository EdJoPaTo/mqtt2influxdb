use crate::floatify::floatify;

#[derive(Debug)]
pub enum Payload {
    String(String),
    Json(serde_json::Value),
    MessagePack(rmpv::Value),
}

impl Payload {
    #[allow(clippy::option_if_let_else)]
    pub fn new(payload: Vec<u8>) -> Option<Self> {
        match String::from_utf8(payload) {
            Ok(payload) => match serde_json::from_str(&payload) {
                Ok(json) => Some(Self::Json(json)),
                Err(_) => Some(Self::String(payload)),
            },
            Err(err) => match rmpv::decode::read_value(&mut err.as_bytes()) {
                Ok(messagepack) => Some(Self::MessagePack(messagepack)),
                Err(_) => None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key<'s> {
    String(&'s str),
    Int(usize),
}
impl std::fmt::Display for Key<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::String(str) => str.fmt(fmt),
            Key::Int(int) => int.fmt(fmt),
        }
    }
}
impl<'s> TryFrom<&'s rmpv::Value> for Key<'s> {
    type Error = ();

    fn try_from(value: &'s rmpv::Value) -> Result<Self, Self::Error> {
        use rmpv::Value;
        fn inner(value: &Value) -> Option<Key> {
            match value {
                Value::Boolean(true) => Some(Key::Int(1)),
                Value::Boolean(false) | Value::Nil => Some(Key::Int(0)),
                Value::Integer(int) => Some(Key::Int(usize::try_from(int.as_u64()?).ok()?)),
                Value::String(string) => Some(Key::String(string.as_str()?)),
                Value::F32(_)
                | Value::F64(_)
                | Value::Binary(_)
                | Value::Array(_)
                | Value::Map(_)
                | Value::Ext(_, _) => None,
            }
        }
        inner(value).ok_or(())
    }
}

#[derive(Debug)]
pub enum Values<'k> {
    Single(f64),
    Many(Vec<(Vec<Key<'k>>, f64)>),
}

impl<'k> Values<'k> {
    pub fn from(payload: &'k Payload) -> Option<Self> {
        let many = match payload {
            Payload::String(payload) => return Some(Self::Single(floatify(payload)?)),
            Payload::Json(payload) => json(payload),
            Payload::MessagePack(payload) => messagepack(payload),
        };
        // Cleanup many
        match many.as_slice() {
            [] => None, // No value
            // single value without key
            [(keys, value)] if keys.is_empty() => Some(Self::Single(*value)),
            _ => Some(Self::Many(many)),
        }
    }
}

pub fn json(value: &serde_json::Value) -> Vec<(Vec<Key<'_>>, f64)> {
    use serde_json::Value;
    fn inner<'json>(
        result: &mut Vec<(Vec<Key<'json>>, f64)>,
        current_key: Vec<Key<'json>>,
        value: &'json Value,
    ) -> Option<()> {
        let simple = match value {
            Value::Null => None,
            Value::Bool(true) => Some(1.0),
            Value::Bool(false) => Some(0.0),
            Value::Number(value) => value.as_f64(),
            Value::String(value) => floatify(value),
            Value::Array(array) => {
                for (index, value) in array.iter().enumerate() {
                    let mut current_key = current_key.clone();
                    current_key.push(Key::Int(index));
                    _ = inner(result, current_key, value);
                }
                None
            }
            Value::Object(object) => {
                for (key, value) in object {
                    let mut current_key = current_key.clone();
                    current_key.push(Key::String(key));
                    _ = inner(result, current_key, value);
                }
                None
            }
        }?;
        result.push((current_key, simple));
        Some(())
    }
    let mut result = Vec::new();
    _ = inner(&mut result, Vec::new(), value);
    result
}

pub fn messagepack(value: &rmpv::Value) -> Vec<(Vec<Key<'_>>, f64)> {
    use rmpv::Value;
    fn inner<'json>(
        result: &mut Vec<(Vec<Key<'json>>, f64)>,
        current_key: Vec<Key<'json>>,
        value: &'json Value,
    ) -> Option<()> {
        let simple = match value {
            Value::Boolean(true) => Some(1.0),
            Value::Boolean(false) => Some(0.0),
            Value::Integer(int) => int.as_f64(),
            Value::F32(float) => Some(f64::from(*float)),
            Value::F64(float) => Some(*float),
            Value::String(str) => floatify(str.as_str()?),
            Value::Array(array) => {
                for (index, value) in array.iter().enumerate() {
                    let mut current_key = current_key.clone();
                    current_key.push(Key::Int(index));
                    _ = inner(result, current_key, value);
                }
                None
            }
            Value::Map(map) => {
                for (key, value) in map {
                    let mut current_key = current_key.clone();
                    current_key.push(key.try_into().ok()?);
                    _ = inner(result, current_key, value);
                }
                None
            }
            Value::Nil | Value::Binary(_) | Value::Ext(_, _) => None,
        }?;
        result.push((current_key, simple));
        Some(())
    }
    let mut result = Vec::new();
    _ = inner(&mut result, Vec::new(), value);
    result
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[track_caller]
    fn single(payload: &Payload) -> f64 {
        match &dbg!(Values::from(payload)).unwrap() {
            Values::Single(value) => *value,
            Values::Many(_) => panic!("not single"),
        }
    }

    #[track_caller]
    fn many<const N: usize>(payload: &Payload) -> [(Vec<Key<'_>>, f64); N] {
        match dbg!(Values::from(payload)).unwrap() {
            Values::Single(_) => panic!("not many"),
            Values::Many(many) => match many.try_into() {
                Ok(many) => many,
                Err(original) => panic!(
                    "different amount of entries. Expected: {N} Actual: {}",
                    original.len()
                ),
            },
        }
    }

    #[test]
    fn payload_parses_string() {
        match dbg!(Payload::new(b"whatever".to_vec())).unwrap() {
            Payload::String(string) => assert_eq!(string, "whatever"),
            Payload::Json(_) | Payload::MessagePack(_) => unreachable!(),
        }
    }

    #[test]
    fn string_values() {
        let value = single(&Payload::String(" 12.3".to_owned()));
        assert_float_eq!(value, 12.3, abs <= 0.001);
    }

    mod json {
        use serde_json::{json, Number, Value};

        use super::*;

        #[test]
        fn plain_number() {
            let payload = Payload::Json(Value::Number(Number::from_f64(12.3).unwrap()));
            let value = single(&payload);
            assert_float_eq!(value, 12.3, abs <= 0.001);
        }

        #[test]
        fn serde_json_macro_example() {
            let payload = Payload::Json(json!({
                "code": 200,
                "success": true,
                "payload": {
                    "features": [
                        "serde",
                        "json"
                    ],
                    "homepage": null
                }
            }));
            let [code, success] = many(&payload);

            let (keys, value) = code;
            assert_eq!(keys, &[Key::String("code")]);
            assert_float_eq!(value, 200.0, abs <= 0.001);

            let (keys, value) = success;
            assert_eq!(keys, &[Key::String("success")]);
            assert_float_eq!(value, 1.0, abs <= 0.001);
        }
    }

    mod messagepack {
        use rmpv::Value;

        use super::*;

        #[test]
        fn parse_works() {
            let value = Value::F64(12.3);
            let mut buffer = Vec::new();
            rmpv::encode::write_value(&mut buffer, &value).unwrap();
            match Payload::new(buffer).unwrap() {
                Payload::MessagePack(Value::F64(value)) => {
                    assert_float_eq!(value, 12.3, abs <= 0.001);
                }
                _ => panic!("unexpected value"),
            }
        }

        #[test]
        fn plain_f64() {
            let value = single(&Payload::MessagePack(Value::F64(12.3)));
            assert_float_eq!(value, 12.3, abs <= 0.001);
        }

        #[test]
        fn similar_to_json_example() {
            let payload = Payload::MessagePack(Value::Map(vec![
                (Value::String("code".into()), Value::Integer(200.into())),
                (Value::String("success".into()), Value::Boolean(true)),
                (
                    Value::String("payload".into()),
                    Value::Map(vec![
                        (
                            Value::String("features".into()),
                            Value::Array(vec![
                                Value::String("serde".into()),
                                Value::String("rmpv".into()),
                            ]),
                        ),
                        (Value::String("homepage".into()), Value::Nil),
                    ]),
                ),
            ]));
            let [code, success] = many(&payload);

            let (keys, value) = code;
            assert_eq!(keys, &[Key::String("code")]);
            assert_float_eq!(value, 200.0, abs <= 0.001);

            let (keys, value) = success;
            assert_eq!(keys, &[Key::String("success")]);
            assert_float_eq!(value, 1.0, abs <= 0.001);
        }
    }
}

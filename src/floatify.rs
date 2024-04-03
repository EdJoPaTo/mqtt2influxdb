/// Assume floats of the input string, otherwise return None
pub fn floatify(input: &str) -> Option<f64> {
    match input {
        "true" | "True" | "TRUE" | "on" | "On" | "ON" | "online" | "Online" | "ONLINE" => Some(1.0),
        "false" | "False" | "FALSE" | "off" | "Off" | "OFF" | "offline" | "Offline" | "OFFLINE" => {
            Some(0.0)
        }
        input => input
            .split(char::is_whitespace)
            .find(|part| !part.is_empty())? // lazy trim
            .parse::<f64>()
            .ok()
            .filter(|float| float.is_finite()),
    }
}

#[cfg(test)]
fn test_some(input: &str, expected: f64) {
    float_eq::assert_float_eq!(floatify(input).unwrap(), expected, abs <= 0.1);
}

#[test]
fn floatify_int() {
    test_some("42", 42.0);
}

#[test]
fn floatify_float() {
    test_some("13.37", 13.37);
}

#[test]
fn floatify_bool() {
    test_some("true", 1.0);
    test_some("false", 0.0);
}

#[test]
fn floatify_on() {
    test_some("on", 1.0);
    test_some("off", 0.0);
}

#[test]
fn floatify_units() {
    test_some("12.3 °C", 12.3);
    test_some(" 12.3 °C", 12.3);
}

#[test]
fn floatify_empty() {
    assert_eq!(floatify(""), None);
    assert_eq!(floatify("  "), None);
}

#[test]
fn floatify_string() {
    assert_eq!(floatify("whatever"), None);
}

#[test]
fn floatify_non_finite() {
    assert_eq!(floatify("nan"), None);
    assert_eq!(floatify("NaN"), None);
    assert_eq!(floatify("inf"), None);
    assert_eq!(floatify("infinity"), None);
}

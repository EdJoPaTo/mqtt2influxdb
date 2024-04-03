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
#[rstest::rstest]
#[case::int("42", 42.0)]
#[case::float("13.37", 13.37)]
#[case::bool("true", 1.0)]
#[case::bool("false", 0.0)]
#[case::on("on", 1.0)]
#[case::on("off", 0.0)]
#[case::units("12.3 °C", 12.3)]
#[case::units(" 12.3 °C", 12.3)]
fn test_some(#[case] input: &str, #[case] expected: f64) {
    float_eq::assert_float_eq!(floatify(dbg!(input)).unwrap(), expected, abs <= 0.001);
}

#[cfg(test)]
#[rstest::rstest]
#[case::empty("")]
#[case::empty("  ")]
#[case::string("whatever")]
#[case::non_finite("nan")]
#[case::non_finite("NaN")]
#[case::non_finite("inf")]
#[case::non_finite("infinity")]
fn test_none(#[case] input: &str) {
    assert_eq!(floatify(dbg!(input)), None);
}

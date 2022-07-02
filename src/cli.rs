use clap::{command, value_parser, Arg, ArgGroup, Command, ValueHint};

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn build() -> Command<'static> {
    command!()
        .arg(
            Arg::new("influx-host")
                .long("influx-host")
                .env("INFLUX_HOST")
                .value_hint(ValueHint::Url)
                .value_name("INFLUX_HOST")
                .value_parser(value_parser!(url::Url))
                .takes_value(true)
                .default_value("http://localhost:8086/")
                .help("HTTP address of InfluxDB"),
        )
        .arg(
            Arg::new("influx-token")
                .long("influx-token")
                .env("INFLUX_TOKEN")
                .value_hint(ValueHint::Other)
                .value_name("TOKEN")
                .hide_env_values(true)
                .takes_value(true)
                .help("InfluxDB api token with write access"),
        )
        .group(ArgGroup::new("influxtarget").args(&["influx-org", "influx-database"]).required(true))
        .arg(
            Arg::new("influx-database")
                .long("influx-database")
                .env("INFLUX_DATABASE")
                .value_hint(ValueHint::Other)
                .value_name("STRING")
                .takes_value(true)
                .conflicts_with_all(&["influx-org", "influx-bucket"])
                .help("InfluxDB 1.x database to put the value in"),
        )
        .arg(
            Arg::new("influx-org")
                .long("influx-org")
                .env("INFLUX_ORG")
                .value_hint(ValueHint::Other)
                .value_name("STRING")
                .takes_value(true)
                .conflicts_with("influx-database")
                .requires("influx-bucket")
                .help("InfluxDB v2 organization"),
        )
        .arg(
            Arg::new("influx-bucket")
                .long("influx-bucket")
                .env("INFLUX_BUCKET")
                .value_hint(ValueHint::Other)
                .value_name("STRING")
                .takes_value(true)
                .conflicts_with("influx-database")
                .requires("influx-org")
                .default_value("mqtt")
                .help("InfluxDB v2 bucket to put the values in"),
        )
        .arg(
            Arg::new("mqtt-broker")
                .long("mqtt-broker")
                .env("MQTT_BROKER")
                .value_hint(ValueHint::Hostname)
                .value_name("HOST")
                .takes_value(true)
                .default_value("localhost")
                .help("Host on which the MQTT Broker is running"),
        )
        .arg(
            Arg::new("mqtt-port")
                .long("mqtt-port")
                .env("MQTT_PORT")
                .value_hint(ValueHint::Other)
                .value_name("INT")
                .value_parser(value_parser!(u16))
                .takes_value(true)
                .default_value("1883")
                .help("Port on which the MQTT Broker is running"),
        )
        .arg(
            Arg::new("mqtt-user")
                .long("mqtt-user")
                .env("MQTT_USERNAME")
                .value_hint(ValueHint::Username)
                .value_name("STRING")
                .takes_value(true)
                .requires("mqtt-password")
                .help("Username to access the MQTT broker")
                .long_help(
                    "Username to access the MQTT broker. Anonymous access when not supplied.",
                ),
        )
        .arg(
            Arg::new("mqtt-password")
                .long("mqtt-password")
                .env("MQTT_PASSWORD")
                .value_hint(ValueHint::Other)
                .value_name("STRING")
                .hide_env_values(true)
                .takes_value(true)
                .requires("mqtt-user")
                .help("Password to access the MQTT broker")
                .long_help(
                    "Password to access the MQTT broker. Passing the password via command line is insecure as the password can be read from the history!",
                ),
        )
        .arg(
            Arg::new("topics")
                .env("MQTT_TOPICS")
                .value_hint(ValueHint::Other)
                .value_name("TOPIC")
                .takes_value(true)
                .multiple_values(true)
                .default_value("#")
                .help("MQTT topics to subscribe"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show more details"),
        )
        .arg(
            Arg::new("buffer-amount")
                .long("buffer-amount")
                .value_hint(ValueHint::Other)
                .value_name("INT")
                .value_parser(value_parser!(usize))
                .takes_value(true)
                .default_value("1000")
                .help("Send the buffer when the amount of messages is reached (or the time)"),
        )
        .arg(
            Arg::new("buffer-seconds")
                .long("buffer-seconds")
                .value_hint(ValueHint::Other)
                .value_name("SECONDS")
                .value_parser(value_parser!(f32))
                .takes_value(true)
                .default_value("28.2")
                .help("Send the buffer when the timeout in seconds has reached (or the amount)"),
        )
}

#[test]
fn verify() {
    build().debug_assert();
}

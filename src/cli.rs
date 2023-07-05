use clap::{ArgGroup, Parser, ValueHint};

#[derive(Debug, Parser)]
#[command(about, version)]
#[command(group(
    ArgGroup::new("influxtarget")
        .args(&["influx_org", "influx_database"])
        .required(true)
))]
pub struct Cli {
    /// HTTP address of InfluxDB
    #[arg(
        long, env,
        value_hint = ValueHint::Url,
        default_value = "http://localhost:8086/"
    )]
    pub influx_host: url::Url,

    /// InfluxDB api token with write access
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "TOKEN",
        hide_env_values = true,
    )]
    pub influx_token: Option<String>,

    /// InfluxDB 1.x database to put the value in
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        conflicts_with_all = &["influx_org", "influx_bucket"],
    )]
    pub influx_database: Option<String>,

    /// InfluxDB v2 organization
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        conflicts_with = "influx_database",
        requires = "influx_bucket",
    )]
    pub influx_org: Option<String>,

    /// InfluxDB v2 organization
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        conflicts_with = "influx_database",
        requires = "influx_org",
    )]
    pub influx_bucket: Option<String>,

    /// Host on which the MQTT Broker is running
    #[arg(
        long, env,
        value_hint = ValueHint::Hostname,
        value_name = "HOST",
        default_value = "localhost",
    )]
    pub mqtt_broker: String,

    /// Port on which the MQTT Broker is running
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "INT",
        value_parser = clap::value_parser!(u16).range(1..),
        default_value = "1883",
    )]
    pub mqtt_port: u16,

    /// Username to access the MQTT broker.
    ///
    /// Anonymous access when not supplied.
    #[arg(
        long, env,
        value_hint = ValueHint::Username,
        value_name = "STRING",
        requires = "mqtt_password",
    )]
    pub mqtt_user: Option<String>,

    /// Password to access the MQTT broker.
    ///
    /// Passing the password via command line is insecure as the password can be read from the history!
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        hide_env_values = true,
        requires = "mqtt_user",
    )]
    pub mqtt_password: Option<String>,

    /// MQTT topics to subscribe
    #[arg(
        env,
        value_hint = ValueHint::Other,
        value_name = "TOPIC",
        default_value = "#",
    )]
    pub mqtt_topics: Vec<String>,

    /// Show more details
    #[arg(short, long)]
    pub verbose: bool,

    /// Send the buffer when the amount of messages is reached (or the time)
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "INT",
        default_value = "1000",
    )]
    pub buffer_amount: usize,

    /// Send the buffer when the timeout in seconds has reached (or the amount)
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "SECONDS",
        default_value = "28.2",
    )]
    pub buffer_seconds: f32,
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}

use clap::{ArgGroup, Parser, ValueHint};

#[derive(Debug, Parser)]
#[clap(about, author, version)]
#[clap(group(
    ArgGroup::new("influxtarget")
        .args(&["influx-org", "influx-database"])
        .required(true)
))]
pub struct Cli {
    /// HTTP address of InfluxDB
    #[clap(
        long, env,
        value_hint = ValueHint::Url,
        default_value = "http://localhost:8086/"
    )]
    pub influx_host: url::Url,

    /// InfluxDB api token with write access
    #[clap(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "TOKEN",
        hide_env_values = true,
    )]
    pub influx_token: Option<String>,

    /// InfluxDB 1.x database to put the value in
    #[clap(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        conflicts_with_all = &["influx-org", "influx-bucket"],
    )]
    pub influx_database: Option<String>,

    /// InfluxDB v2 organization
    #[clap(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        conflicts_with = "influx-database",
    )]
    pub influx_org: Option<String>,

    /// InfluxDB v2 organization
    #[clap(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        conflicts_with = "influx-database",
    )]
    pub influx_bucket: Option<String>,

    /// Host on which the MQTT Broker is running
    #[clap(
        long, env,
        value_hint = ValueHint::Hostname,
        value_name = "HOST",
        default_value = "localhost",
    )]
    pub mqtt_broker: String,

    /// Port on which the MQTT Broker is running
    #[clap(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "INT",
        default_value = "1883",
    )]
    pub mqtt_port: u16,

    /// Username to access the MQTT broker.
    ///
    /// Anonymous access when not supplied.
    #[clap(
        long, env,
        value_hint = ValueHint::Username,
        value_name = "STRING",
        requires = "mqtt-password",
    )]
    pub mqtt_user: Option<String>,

    /// Password to access the MQTT broker.
    ///
    /// Passing the password via command line is insecure as the password can be read from the history!
    #[clap(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        hide_env_values = true,
        requires = "mqtt-user",
    )]
    pub mqtt_password: Option<String>,

    /// MQTT topics to subscribe
    #[clap(
        env,
        value_hint = ValueHint::Other,
        value_name = "TOPIC",
        default_value = "#",
    )]
    pub mqtt_topics: Vec<String>,

    /// Show more details
    #[clap(short, long)]
    pub verbose: bool,

    /// Send the buffer when the amount of messages is reached (or the time)
    #[clap(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "INT",
        default_value = "1000",
    )]
    pub buffer_amount: usize,

    /// Send the buffer when the timeout in seconds has reached (or the amount)
    #[clap(
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

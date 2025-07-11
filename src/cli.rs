use clap::{ArgGroup, Parser, ValueHint};

#[expect(clippy::doc_markdown)]
#[derive(Debug, Parser)]
#[command(about, version)]
#[command(group(
    ArgGroup::new("influxtarget")
        .args(&["influx_org", "influx_database", "victoria_metrics"])
        .required(true)
))]
pub struct Cli {
    /// HTTP address of InfluxDB
    #[arg(
        long, env,
        value_hint = ValueHint::Url,
        help_heading = "Database",
        default_value = "http://localhost:8086/"
    )]
    pub influx_host: url::Url,

    /// InfluxDB API token with write access
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "TOKEN",
        help_heading = "Database",
        hide_env_values = true,
    )]
    pub influx_token: Option<String>,

    /// InfluxDB 1.x database to put the value in
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        help_heading = "Database",
        conflicts_with_all = &["influx_org", "influx_bucket"],
    )]
    pub influx_database: Option<String>,

    /// InfluxDB v2 organization
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        help_heading = "Database",
        conflicts_with = "influx_database",
        requires = "influx_bucket",
    )]
    pub influx_org: Option<String>,

    /// InfluxDB v2 organization
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "STRING",
        help_heading = "Database",
        conflicts_with = "influx_database",
        requires = "influx_org",
    )]
    pub influx_bucket: Option<String>,

    /// VictoriaMetrics doesn't need database, organization or bucket
    #[arg(
        long, env,
        help_heading = "Database",
        conflicts_with_all = &["influx_database", "influx_org", "influx_bucket"],
    )]
    pub victoria_metrics: bool,

    /// Host on which the MQTT Broker is running
    #[arg(
        long, env,
        value_hint = ValueHint::Hostname,
        value_name = "HOST",
        help_heading = "MQTT",
        default_value = "localhost",
    )]
    pub mqtt_broker: String,

    /// Port on which the MQTT Broker is running
    #[arg(
        long, env,
        value_hint = ValueHint::Other,
        value_name = "INT",
        help_heading = "MQTT",
        default_value = "1883",
    )]
    pub mqtt_port: std::num::NonZeroU16,

    /// Username to access the MQTT broker.
    ///
    /// Anonymous access when not supplied.
    #[arg(
        long, env,
        value_hint = ValueHint::Username,
        value_name = "STRING",
        help_heading = "MQTT",
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
        help_heading = "MQTT",
        hide_env_values = true,
        requires = "mqtt_user",
    )]
    pub mqtt_password: Option<String>,

    /// MQTT topics to subscribe
    #[arg(
        env,
        value_hint = ValueHint::Other,
        value_name = "TOPIC",
        help_heading = "MQTT",
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
    use clap::CommandFactory as _;
    Cli::command().debug_assert();
}

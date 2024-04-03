use std::time::Duration;

use clap::Parser;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;

mod cli;
mod exit_handler;
mod floatify;
mod influxdb;
mod message;
mod mqtt;
mod payload;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let matches = cli::Cli::parse();

    let mut influxdb = influxdb::Influxdb::new(
        matches.influx_host,
        matches.influx_token.as_deref(),
        matches.influx_database.as_deref(),
        matches.influx_org.as_deref(),
        matches.influx_bucket.as_deref(),
        Duration::from_secs_f32(matches.buffer_seconds),
        matches.buffer_amount,
    )
    .await;
    eprintln!("InfluxDB connected: {}", influxdb.get_write_url());

    let mqtt_broker = matches.mqtt_broker;
    let (client, mut receiver) = mqtt::connect(
        &mqtt_broker,
        matches.mqtt_port,
        matches.mqtt_user.as_deref(),
        matches.mqtt_password.as_deref(),
        matches.mqtt_topics,
        matches.verbose,
    )
    .await;
    eprintln!("MQTT {mqtt_broker} connected.");

    let quit = exit_handler::ExitHandler::new();

    eprintln!("Startup done. Listening to topics now…");

    let mut error = false;
    loop {
        if quit.is_exiting() {
            client
                .disconnect()
                .await
                .expect("failed to disconnect MQTT");
            break;
        }

        match receiver.try_recv() {
            Ok(message) => influxdb.append(message.into_line_protocol()),
            Err(TryRecvError::Empty) => sleep(Duration::from_millis(50)).await,
            Err(TryRecvError::Disconnected) => {
                eprintln!("MQTT sender is gone");
                error = true;
                break;
            }
        }
        influxdb.do_loop().await;
    }

    while let Some(message) = receiver.recv().await {
        influxdb.append(message.into_line_protocol());
    }
    influxdb.async_drop().await;

    if error {
        std::process::exit(-1);
    }
}

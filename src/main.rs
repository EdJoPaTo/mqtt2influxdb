use std::time::Duration;

use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;
use url::Url;

mod cli;
mod exit_handler;
mod influxdb;
mod message;
mod mqtt;

#[tokio::main]
async fn main() {
    let matches = cli::build().get_matches();

    let buffer_amount = *matches.get_one::<usize>("buffer-amount").unwrap();
    let buffer_age = matches
        .get_one::<f32>("buffer-seconds")
        .copied()
        .map(Duration::from_secs_f32)
        .unwrap();

    let topics = matches
        .get_many::<String>("topics")
        .unwrap()
        .map(std::string::ToString::to_string)
        .collect();

    let influx_host = matches.get_one::<Url>("influx-host").unwrap();
    let mut influxdb = influxdb::Influxdb::new(
        influx_host,
        matches
            .get_one::<String>("influx-token")
            .map(String::as_str),
        matches
            .get_one::<String>("influx-database")
            .map(String::as_str),
        matches.get_one::<String>("influx-org").map(String::as_str),
        matches
            .get_one::<String>("influx-bucket")
            .map(String::as_str),
        buffer_age,
        buffer_amount,
    )
    .await;
    eprintln!("InfluxDB {} connected.", influx_host);

    let mqtt_broker = matches.get_one::<String>("mqtt-broker").unwrap();
    let (client, mut receiver) = mqtt::connect(
        mqtt_broker,
        *matches.get_one("mqtt-port").unwrap(),
        matches.get_one::<String>("mqtt-user").map(String::as_str),
        matches
            .get_one::<String>("mqtt-password")
            .map(String::as_str),
        topics,
        matches.contains_id("verbose"),
    )
    .await;
    eprintln!("MQTT {} connected.", mqtt_broker);

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
            Ok(message) => {
                if let Some(line) = message.into_line_protocol() {
                    influxdb.push(line);
                }
            }
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
        if let Some(line) = message.into_line_protocol() {
            influxdb.push(line);
        }
    }
    influxdb.async_drop().await;

    if error {
        std::process::exit(-1);
    }
}

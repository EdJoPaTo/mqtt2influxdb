use std::time::Duration;

use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;

mod cli;
mod exit_handler;
mod influxdb;
mod message;
mod mqtt;

#[tokio::main]
async fn main() {
    let matches = cli::build().get_matches();

    let buffer_amount = matches
        .value_of("buffer-amount")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap();
    let buffer_age = matches
        .value_of("buffer-seconds")
        .and_then(|s| s.parse::<f32>().ok())
        .map(Duration::from_secs_f32)
        .unwrap();

    let topics = matches
        .values_of("topics")
        .unwrap()
        .map(std::string::ToString::to_string)
        .collect();

    let influx_host = &matches.value_of("influx-host").unwrap();
    let mut influxdb = influxdb::Influxdb::new(
        influx_host,
        matches.value_of("influx-token"),
        matches.value_of("influx-database"),
        matches.value_of("influx-org"),
        matches.value_of("influx-bucket"),
        buffer_age,
        buffer_amount,
    )
    .await;
    eprintln!("InfluxDB {} connected.", influx_host);

    let mqtt_broker = &matches.value_of("mqtt-broker").unwrap();
    let (client, mut receiver) = mqtt::connect(
        mqtt_broker,
        matches
            .value_of("mqtt-port")
            .and_then(|s| s.parse().ok())
            .unwrap(),
        matches.value_of("mqtt-user"),
        matches.value_of("mqtt-password"),
        topics,
        matches.is_present("verbose"),
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

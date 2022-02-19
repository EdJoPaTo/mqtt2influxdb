use std::time::Duration;

use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;

use crate::linebuffer::LineBuffer;

mod cli;
mod influxdb;
mod linebuffer;
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
    let influxdb = influxdb::Influxdb::new(
        influx_host,
        matches.value_of("influx-token"),
        matches.value_of("influx-database"),
        matches.value_of("influx-org"),
        matches.value_of("influx-bucket"),
    )
    .await;
    eprintln!("InfluxDB {} connected.", influx_host);

    let mqtt_broker = &matches.value_of("mqtt-broker").unwrap();
    let mut receiver = mqtt::connect(
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

    let mut linebuffer = LineBuffer::new(buffer_age, buffer_amount);
    eprintln!("Startup done. Listening to topics now…");

    loop {
        match receiver.try_recv() {
            Ok(message) => {
                if let Some(line) = message.into_line_protocol() {
                    linebuffer.push(line);
                }
            }
            Err(TryRecvError::Empty) => sleep(Duration::from_millis(50)).await,
            Err(TryRecvError::Disconnected) => panic!("MQTT sender is gone"),
        }
        linebuffer.write(&influxdb).await;
    }
}

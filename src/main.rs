use std::time::{Duration, Instant};

use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;

use crate::influxdb::Influxdb;

mod cli;
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

    let influxdb = influxdb::Influxdb::new(
        matches.value_of("influx-host").unwrap(),
        matches.value_of("influx-token"),
        matches.value_of("influx-database"),
        matches.value_of("influx-org"),
        matches.value_of("influx-bucket"),
    )
    .await;
    eprintln!("InfluxDB connected.");

    let mut receiver = mqtt::connect(
        matches.value_of("mqtt-broker").unwrap(),
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
    eprintln!("MQTT connected.");

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

struct LineBuffer {
    max_age: Duration,
    max_amount: usize,

    buffer: Vec<String>,
    last_send: Instant,
    next_error_millis: u64,
}

impl LineBuffer {
    pub fn new(max_age: Duration, max_amount: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_amount,
            max_age,
            last_send: Instant::now(),
            next_error_millis: 8,
        }
    }

    pub fn push(&mut self, line: String) {
        self.buffer.push(line);
    }

    pub async fn write(&mut self, influxdb: &Influxdb) {
        if self.buffer.is_empty() {
            return;
        }
        if self.buffer.len() >= self.max_amount || self.last_send.elapsed() > self.max_age {
            if let Err(err) = influxdb.write(&self.buffer).await {
                eprintln!("InfluxDB write failed {}", err);
                sleep(Duration::from_millis(self.next_error_millis)).await;
                self.next_error_millis *= 2;
                self.next_error_millis = self.next_error_millis.min(30_000); // Up to 30 seconds
            } else {
                self.last_send = Instant::now();
                println!("sent {} lines", self.buffer.len());
                self.buffer.clear();
                self.next_error_millis = 8;
            }
        }
    }
}

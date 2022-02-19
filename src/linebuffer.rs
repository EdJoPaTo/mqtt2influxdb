use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::influxdb::Influxdb;

pub struct LineBuffer {
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

use std::time::{Duration, Instant};

use reqwest::header;
use tokio::time::sleep;
use url::Url;

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY"),
);

pub struct Influxdb {
    write_url: Url,
    client: reqwest::Client,
    error_count: u64,

    last_send: Instant,
    max_age: Duration,

    linebuffer: Vec<String>,
    max_amount: usize,
}

impl Influxdb {
    pub async fn new(
        host: &str,
        api_token: Option<&str>,
        database: Option<&str>,
        org: Option<&str>,
        bucket: Option<&str>,
        max_age: Duration,
        max_amount: usize,
    ) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(token) = api_token {
            let mut auth_value = header::HeaderValue::from_str(&format!("Token {}", token))
                .expect("InfluxDB API_TOKEN is no valid HTTP Header");
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .timeout(Duration::from_secs(5))
            .user_agent(USER_AGENT)
            .build()
            .unwrap();

        let mut url = Url::parse(host).unwrap();
        if let Some(database) = database {
            url.set_path("/write");
            url.set_query(Some(&format!("db={}", database)));
        } else {
            url.set_path("/api/v2/write");
            url.set_query(Some(&format!(
                "org={}&bucket={}",
                org.unwrap(),
                bucket.unwrap()
            )));
        }

        if let Err(err) = write(&client, url.as_str(), &[]).await {
            panic!("failed InfluxDB test-write: {}", err);
        }

        Self {
            write_url: url,
            client,
            error_count: 0,

            last_send: Instant::now(),
            max_age,

            linebuffer: Vec::with_capacity(max_amount),
            max_amount,
        }
    }

    pub fn push(&mut self, line: String) {
        self.linebuffer.push(line);
    }

    async fn write(&mut self) -> anyhow::Result<()> {
        write(&self.client, self.write_url.as_str(), &self.linebuffer).await?;
        self.last_send = Instant::now();
        println!("sent {} lines", self.linebuffer.len());
        self.linebuffer.clear();
        Ok(())
    }

    pub async fn do_loop(&mut self) {
        if self.linebuffer.len() >= self.max_amount || self.last_send.elapsed() > self.max_age {
            if let Err(err) = self.write().await {
                self.error_count += 1;
                eprintln!(
                    "InfluxDB write failed (error_count: {}): {}",
                    self.error_count, err
                );
                let error_millis = (self.error_count * 91).min(30_000); // Up to 30
                sleep(Duration::from_millis(error_millis)).await;
            } else {
                self.error_count = 0;
            }
        }
    }

    /// This is a workaround as impl Drop for Influxdb cant do something async
    pub async fn async_drop(&mut self) {
        self.write()
            .await
            .expect("failed to write final buffer content to InfluxDB");
    }
}

async fn write(client: &reqwest::Client, url: &str, lines: &[String]) -> anyhow::Result<()> {
    let result = client.post(url).body(lines.join("\n")).send().await?;
    let status = result.status();
    if status.is_client_error() || status.is_server_error() {
        Err(if let Ok(text) = result.text().await {
            anyhow::anyhow!("InfluxDB write error: {:?}", text)
        } else {
            anyhow::anyhow!("Unknown InfluxDB write error")
        })
    } else {
        Ok(())
    }
}

impl Drop for Influxdb {
    /// use `Influxdb::async_drop` manually
    fn drop(&mut self) {
        assert!(
            self.linebuffer.is_empty(),
            "InfluxDB is dropped with {} unsent lines in buffer!",
            self.linebuffer.len()
        );
    }
}

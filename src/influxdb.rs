use std::time::{Duration, Instant};

use anyhow::Context as _;
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
    verbose: bool,

    last_send: Instant,
    max_age: Duration,

    linebuffer: Vec<String>,
    max_amount: usize,
}

impl Influxdb {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        host: Url,
        api_token: Option<&str>,
        database: Option<&str>,
        org: Option<&str>,
        bucket: Option<&str>,
        max_age: Duration,
        max_amount: usize,
        verbose: bool,
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
            let mut auth_value = header::HeaderValue::from_str(&format!("Token {token}"))
                .expect("InfluxDB API_TOKEN is no valid HTTP Header");
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .timeout(Duration::from_secs(5))
            .user_agent(header::HeaderValue::from_static(USER_AGENT))
            .build()
            .unwrap();

        let mut url = host;
        if let (Some(org), Some(bucket)) = (org, bucket) {
            url.set_path("/api/v2/write");
            url.set_query(Some(&format!("org={org}&bucket={bucket}")));
        } else if let Some(database) = database {
            url.set_path("/write");
            url.set_query(Some(&format!("db={database}")));
        } else {
            url.set_path("/write");
        }

        if let Err(err) = write(&client, url.clone(), &[]).await {
            panic!("failed InfluxDB test-write: {err:?}");
        }

        Self {
            write_url: url,
            client,
            error_count: 0,
            verbose,

            last_send: Instant::now(),
            max_age,

            linebuffer: Vec::with_capacity(max_amount),
            max_amount,
        }
    }

    pub const fn get_write_url(&self) -> &Url {
        &self.write_url
    }

    /// Append to the lines that will be written
    pub fn append(&mut self, mut lines: Vec<String>) {
        if self.verbose {
            for line in &lines {
                println!("InfluxDB Line: {line}");
            }
        }
        self.linebuffer.append(&mut lines);
    }

    async fn write(&mut self) -> anyhow::Result<()> {
        write(&self.client, self.write_url.clone(), &self.linebuffer).await?;
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
                    "InfluxDB write failed (error_count: {}): {err:#}",
                    self.error_count
                );
                let error_millis = (self.error_count * 91).min(30_000); // Up to 30
                sleep(Duration::from_millis(error_millis)).await;
            } else {
                self.error_count = 0;
            }
        }
    }

    /// This is a workaround as `impl Drop for Influxdb` can't do something async
    pub async fn async_drop(&mut self) {
        self.write()
            .await
            .expect("failed to write final buffer content to InfluxDB");
    }
}

async fn write(client: &reqwest::Client, url: Url, lines: &[String]) -> anyhow::Result<()> {
    let result = client
        .post(url)
        .body(lines.join("\n"))
        .send()
        .await
        .context("Could not send HTTP request")?;
    let status = result.status();
    if status.is_client_error() || status.is_server_error() {
        let reason = result
            .text()
            .await
            .context("Could not get reason from error response body")?;
        anyhow::bail!("InfluxDB specified reason ({status}): {reason}");
    }
    Ok(())
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

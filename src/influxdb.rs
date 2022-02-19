use std::time::Duration;

use reqwest::header;
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
}

impl Influxdb {
    pub async fn new(
        host: &str,
        api_token: Option<&str>,
        database: Option<&str>,
        org: Option<&str>,
        bucket: Option<&str>,
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
            .timeout(Duration::from_secs(1))
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

        let influxdb = Self {
            write_url: url,
            client,
        };
        if let Err(err) = influxdb.write(&[]).await {
            panic!("failed InfluxDB test-write: {}", err);
        }
        influxdb
    }

    pub async fn write(&self, lines: &[String]) -> Result<(), reqwest::Error> {
        self.client
            .post(self.write_url.as_str())
            .body(lines.join("\n"))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

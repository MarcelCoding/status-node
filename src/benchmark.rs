use std::time::Duration;

use anyhow::anyhow;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, StatusCode};
use tokio::time::Instant;

use crate::backend::Service;

#[derive(Debug)]
pub struct Benchmark {
    pub initial: Status,
    pub alive: Status,
}

#[derive(Debug)]
pub struct Status {
    pub code: Option<StatusCode>,
    pub ping: u64,
}

pub async fn execute(sleep: Duration, service: &Service) -> anyhow::Result<Benchmark> {
    let client = Client::builder()
        .timeout(Duration::from_secs(service.timeout as u64))
        .redirect(Policy::none())
        .build()?;

    let initial = ping(&client, service.method.clone(), &service.url).await?;
    tokio::time::sleep(sleep).await;
    let alive = ping(&client, service.method.clone(), &service.url).await?;
    tokio::time::sleep(sleep).await;

    if initial.code != alive.code {
        return Err(anyhow!("Different status codes for {}", service.id));
    }

    Ok(Benchmark { initial, alive })
}

async fn ping(client: &Client, method: Method, url: &String) -> anyhow::Result<Status> {
    let request = client.request(method, url).send();

    let start = Instant::now();
    let response = request.await;
    let time = start.elapsed();

    let code = match response {
        Ok(response) => Some(response.status()),
        Err(error) => error.status(),
    };

    Ok(Status {
        code,
        ping: time.as_millis() as u64,
    })
}

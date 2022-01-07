use std::time::{Duration, SystemTime};

use reqwest::{Client, Method, StatusCode};

#[derive(Debug)]
pub struct Benchmark {
    initial: Status,
    alive: Status,
}

#[derive(Debug)]
pub struct Status {
    status: Option<StatusCode>,
    ping: u128,
}

pub async fn execute(sleep: Duration, method: Method, url: String) -> anyhow::Result<Benchmark> {
    let client = Client::new();

    let initial = ping(&client, method.clone(), &url).await?;
    tokio::time::sleep(sleep).await;
    let alive = ping(&client, method.clone(), &url).await?;
    tokio::time::sleep(sleep).await;

    Ok(Benchmark { initial, alive })
}

pub async fn verify() -> anyhow::Result<()> {
    let client = Client::new();

    let response = client.head("https://www.cloudflare.com/")
        .send()
        .await?;

    response.error_for_status()?;

    Ok(())
}

async fn ping(client: &Client, method: Method, url: &String) -> anyhow::Result<Status> {
    let request = client.request(method, url).send();

    let start = SystemTime::now();
    let response = request.await;
    let time = start.elapsed()?;

    let status = match response {
        Ok(response) => Some(response.status()),
        Err(error) => error.status(),
    };

    Ok(Status {
        status,
        ping: time.as_millis(),
    })
}

use std::io::{stdout, Write};
use std::time::Duration;

use reqwest::Method;

use crate::benchmark::{execute, verify};

mod benchmark;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let urls = vec![
        "https://overleaf.secshell.net/".to_string(),
        "https://www.google.com/".to_string(),
        "https://element.m4rc3l.de/".to_string(),
    ];

    print!("Verifying...");
    stdout().flush()?;
    verify().await?;
    println!(" done.");

    for url in urls {
        print!("Benchmarking \"{}\"...", url);
        stdout().flush()?;
        let benchmark = execute(Duration::from_millis(500), Method::HEAD, url).await?;
        println!(" done.");
        println!("{:?}", benchmark);
    }

    Ok(())
}


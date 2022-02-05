use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub backend: String,
    pub token: Option<String>,
    pub location: Option<String>,
    pub cache: Option<String>,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let p = path.as_ref();

        if !p.exists() {
            let default = Config::default().await?;
            let data = serde_yaml::to_vec(&default)?;

            let mut file = File::create(p).await?;
            file.write_all(&data).await?;

            return Ok(default);
        }

        let mut file = File::open(p).await?;

        let mut data = String::new();
        file.read_to_string(&mut data).await?;

        let config = serde_yaml::from_str(data.as_str())?;

        Ok(config)
    }

    async fn default() -> anyhow::Result<Self> {
        let location = match find_location().await {
            Ok(option) => option,
            Err(_) => None
        };

        Ok(Config {
            backend: "https://status.m4rc3l.de".to_string(),
            token: None,
            location,
            cache: None,
        })
    }
}

async fn find_location() -> anyhow::Result<Option<String>> {
    let location = reqwest::get("https://ifconfig.co/city")
        .await?
        .error_for_status()?
        .text()
        .await?
        .trim()
        .to_string();

    return if location.is_empty() {
        Ok(None)
    } else {
        Ok(Some(location))
    };
}

use std::path::{Path, PathBuf};

use csv_async::{AsyncDeserializer, AsyncReaderBuilder, AsyncSerializer, AsyncWriterBuilder};
use tokio::fs::{File, OpenOptions};
use tokio_stream::StreamExt;

use crate::cache::{Cache, is_old_enough};
use crate::Ping;

pub struct FileCache {
    path: PathBuf,
}

impl FileCache {
    pub fn new(path: String) -> Self {
        FileCache { path: Path::new(&path).to_owned() }
    }

    async fn open_reader(&self) -> anyhow::Result<AsyncDeserializer<File>> {
        let file = File::open(&self.path).await?;

        let reader = AsyncReaderBuilder::new()
            .has_headers(false)
            .create_deserializer(file);

        Ok(reader)
    }

    async fn open_append_writer(&self) -> anyhow::Result<AsyncSerializer<File>> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)
            .await?;

        let writer = AsyncWriterBuilder::new()
            .has_headers(false)
            .create_serializer(file);

        Ok(writer)
    }
}

#[async_trait::async_trait]
impl Cache for FileCache {
    async fn push(&self, pings: &Vec<Ping>) -> anyhow::Result<()> {
        let writer = &mut self.open_append_writer().await?;

        for ping in pings {
            writer.serialize(ping).await?;
        }

        Ok(())
    }

    async fn read_if_old_enough(&self) -> anyhow::Result<Vec<Ping>> {
        let mut pings = Vec::new();

        if !self.path.exists() {
            return Ok(pings);
        }

        let mut reader = self.open_reader().await?;
        let mut stream = reader.deserialize::<Ping>();

        let mut first = true;

        while let Some(ping) = stream.next().await {
            let ping = ping?;

            if first {
                if !is_old_enough(ping.time) {
                    break;
                }

                first = false;
                pings.push(ping);
            } else {
                pings.push(ping);
            }
        }

        Ok(pings)
    }

    async fn truncate(&self) -> anyhow::Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        File::create(&self.path).await?;

        Ok(())
    }
}

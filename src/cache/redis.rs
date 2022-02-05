use redis::{Client, IntoConnectionInfo};

use crate::{Cache, Ping};

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new<T: IntoConnectionInfo>(conn_info: T) -> anyhow::Result<Self> {
        let client = Client::open(conn_info)?;
        Ok(RedisCache { client })
    }
}

#[async_trait::async_trait]
impl Cache for RedisCache {
    async fn push(&self, pings: &Vec<Ping>) -> anyhow::Result<()> {
        todo!()
    }

    async fn read_if_old_enough(&self) -> anyhow::Result<Vec<Ping>> {
        todo!()
    }

    async fn truncate(&self) -> anyhow::Result<()> {
        todo!()
    }
}

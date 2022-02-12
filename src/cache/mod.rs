use chrono::Utc;

pub use crate::cache::file::FileCache;
pub use crate::cache::redis::RedisCache;
use crate::Ping;

mod file;
mod redis;

const MAX_AGE: u64 = 60 * 30; // 1/2 hour in seconds

#[async_trait::async_trait]
pub trait Cache {
    async fn push(&self, pings: &[Ping]) -> anyhow::Result<()>;
    async fn read_if_old_enough(&self) -> anyhow::Result<Vec<Ping>>;
    async fn truncate(&self) -> anyhow::Result<()>;
}

fn is_old_enough(time: u64) -> bool {
    (Utc::now().timestamp() as u64) - time > MAX_AGE
}

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[cfg(feature = "standalone")]
pub mod redis;

#[cfg(feature = "cloudflare")]
pub mod cf_kv;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub username: String,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[cfg(feature = "standalone")]
    #[error("redis pool error: {0}")]
    Pool(#[from] deadpool_redis::PoolError),
    #[cfg(feature = "standalone")]
    #[error("redis error: {0}")]
    Redis(#[from] ::redis::RedisError),
    #[error("backend error: {0}")]
    Backend(String),
}

#[cfg_attr(feature = "standalone", async_trait::async_trait)]
#[cfg_attr(feature = "cloudflare", async_trait::async_trait(?Send))]
pub trait SessionStore {
    async fn create(
        &self,
        id: &SessionId,
        data: &SessionData,
        ttl: Duration,
    ) -> Result<(), SessionError>;

    async fn load(&self, id: &SessionId) -> Result<Option<SessionData>, SessionError>;

    async fn destroy(&self, id: &SessionId) -> Result<(), SessionError>;
}

pub fn new_session_id() -> SessionId {
    use rand::RngCore;
    let mut bytes = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut bytes);
    SessionId(bs58::encode(bytes).into_string())
}

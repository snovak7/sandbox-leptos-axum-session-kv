use super::{SessionData, SessionError, SessionId, SessionStore};
use deadpool_redis::Pool;
use redis::AsyncCommands;
use std::time::Duration;

#[derive(Clone)]
pub struct RedisSessionStore {
    pub pool: Pool,
}

impl RedisSessionStore {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    fn key(id: &SessionId) -> String {
        format!("session:{}", id.0)
    }
}

#[async_trait::async_trait]
impl SessionStore for RedisSessionStore {
    async fn create(
        &self,
        id: &SessionId,
        data: &SessionData,
        ttl: Duration,
    ) -> Result<(), SessionError> {
        let mut conn = self.pool.get().await?;
        let json = serde_json::to_string(data)?;
        let ttl_secs = ttl.as_secs();
        conn.set_ex::<_, _, ()>(Self::key(id), json, ttl_secs)
            .await?;
        Ok(())
    }

    async fn load(&self, id: &SessionId) -> Result<Option<SessionData>, SessionError> {
        let mut conn = self.pool.get().await?;
        let maybe: Option<String> = conn.get(Self::key(id)).await?;
        match maybe {
            None => Ok(None),
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        }
    }

    async fn destroy(&self, id: &SessionId) -> Result<(), SessionError> {
        let mut conn = self.pool.get().await?;
        conn.del::<_, ()>(Self::key(id)).await?;
        Ok(())
    }
}

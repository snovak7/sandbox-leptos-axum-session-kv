use super::{SessionData, SessionError, SessionId, SessionStore};
use std::time::Duration;
use worker::kv::KvStore;

pub struct CloudflareKvStore {
    pub kv: KvStore,
}

impl CloudflareKvStore {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }

    fn key(id: &SessionId) -> String {
        format!("session:{}", id.0)
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for CloudflareKvStore {
    async fn create(
        &self,
        id: &SessionId,
        data: &SessionData,
        ttl: Duration,
    ) -> Result<(), SessionError> {
        let json = serde_json::to_string(data)?;
        self.kv
            .put(&Self::key(id), json)
            .map_err(|e| SessionError::Backend(e.to_string()))?
            .expiration_ttl(ttl.as_secs())
            .execute()
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;
        Ok(())
    }

    async fn load(&self, id: &SessionId) -> Result<Option<SessionData>, SessionError> {
        let maybe = self
            .kv
            .get(&Self::key(id))
            .text()
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;
        match maybe {
            None => Ok(None),
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        }
    }

    async fn destroy(&self, id: &SessionId) -> Result<(), SessionError> {
        self.kv
            .delete(&Self::key(id))
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;
        Ok(())
    }
}

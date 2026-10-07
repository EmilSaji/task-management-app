//! Per-user Redis cache for `GET /tasks/view-my-tasks`.
//!
//! Key: `{prefix}tasks:user:{user_id}` -> JSON of the response, with a TTL.
//! Writes that change which tasks a user sees (assign / update) call
//! [`TaskCache::invalidate`] for every affected user after the DB commit.

use redis::{aio::ConnectionManager, AsyncCommands};
use uuid::Uuid;

use crate::{dto::task::MyTasksResponse, error::AppResult};

#[derive(Clone)]
pub struct TaskCache {
    conn: ConnectionManager,
    prefix: String,
    ttl_seconds: u64,
}

impl TaskCache {
    pub async fn connect(
        url: &str,
        prefix: String,
        ttl_seconds: u64,
    ) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self {
            conn,
            prefix,
            ttl_seconds,
        })
    }

    fn key(&self, user_id: Uuid) -> String {
        format!("{}tasks:user:{}", self.prefix, user_id)
    }

    pub async fn get_my_tasks(&self, user_id: Uuid) -> AppResult<Option<MyTasksResponse>> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(self.key(user_id)).await?;
        Ok(raw.and_then(|json| match serde_json::from_str(&json) {
            Ok(value) => Some(value),
            Err(e) => {
                // A shape change between deploys: treat as a miss and overwrite.
                tracing::warn!(%user_id, error = %e, "discarding undecodable cache entry");
                None
            }
        }))
    }

    pub async fn set_my_tasks(&self, user_id: Uuid, value: &MyTasksResponse) -> AppResult<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| crate::error::AppError::Internal(format!("cache serialize: {e}")))?;
        let mut conn = self.conn.clone();
        let _: () = conn
            .set_ex(self.key(user_id), json, self.ttl_seconds)
            .await?;
        Ok(())
    }

    pub async fn invalidate(&self, user_ids: &[Uuid]) -> AppResult<()> {
        if user_ids.is_empty() {
            return Ok(());
        }
        let keys: Vec<String> = user_ids.iter().map(|id| self.key(*id)).collect();
        let mut conn = self.conn.clone();
        let _: () = conn.del(keys).await?;
        tracing::debug!(?user_ids, "invalidated task cache");
        Ok(())
    }

    /// Delete every task-cache key under this prefix (used by the dev seed reset).
    pub async fn clear_all(&self) -> AppResult<()> {
        let pattern = format!("{}tasks:user:*", self.prefix);
        let mut conn = self.conn.clone();
        let keys: Vec<String> = {
            let mut iter = conn.scan_match::<_, String>(&pattern).await?;
            let mut keys = Vec::new();
            while let Some(key) = iter.next_item().await {
                keys.push(key);
            }
            keys
        };
        if !keys.is_empty() {
            let mut conn = self.conn.clone();
            let _: () = conn.del(keys).await?;
        }
        Ok(())
    }
}

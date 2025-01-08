use crate::common::error::AppResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
#[allow(dead_code)]
pub struct CacheConnection {
    store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl CacheConnection {
    #[allow(dead_code)]
    pub async fn new() -> AppResult<Self> {
        Ok(Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    #[allow(dead_code)]
    pub async fn get(&self, key: &str) -> AppResult<Option<Vec<u8>>> {
        let store = self.store.read().await;
        Ok(store.get(key).cloned())
    }

    #[allow(dead_code)]
    pub async fn set(&self, key: String, value: Vec<u8>) -> AppResult<()> {
        let mut store = self.store.write().await;
        store.insert(key, value);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_miss() -> AppResult<()> {
        let cache = CacheConnection::new().await?;
        let result = cache.get("nonexistent-key").await?;
        assert!(result.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_cache_set_and_get() -> AppResult<()> {
        let cache = CacheConnection::new().await?;
        let key = "test-key".to_string();
        let value = vec![1, 2, 3];

        cache.set(key.clone(), value.clone()).await?;
        let get_result = cache.get(&key).await?;
        assert_eq!(get_result, Some(value));
        Ok(())
    }

    #[tokio::test]
    async fn test_cache_delete() -> AppResult<()> {
        let cache = CacheConnection::new().await?;
        let key = "test-key".to_string();
        let value = vec![1, 2, 3];

        cache.set(key.clone(), value).await?;
        cache.delete(&key).await?;
        let get_result = cache.get(&key).await?;
        assert!(get_result.is_none());
        Ok(())
    }
}

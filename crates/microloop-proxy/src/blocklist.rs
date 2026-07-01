use async_trait::async_trait;
use redis::AsyncCommands;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait BlocklistStore: Send + Sync {
    async fn add_block(&self, session_id: &str, tool_call: &str) -> Result<(), String>;
    async fn is_blocked(&self, session_id: &str, tool_call: &str) -> Result<bool, String>;
}

pub struct InMemoryBlocklist {
    store: Arc<Mutex<HashMap<String, HashSet<String>>>>,
}

impl InMemoryBlocklist {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl BlocklistStore for InMemoryBlocklist {
    async fn add_block(&self, session_id: &str, tool_call: &str) -> Result<(), String> {
        let mut blocklist = self.store.lock().unwrap();
        blocklist
            .entry(session_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(tool_call.to_string());
        Ok(())
    }

    async fn is_blocked(&self, session_id: &str, tool_call: &str) -> Result<bool, String> {
        let blocklist = self.store.lock().unwrap();
        if let Some(session_blocks) = blocklist.get(session_id) {
            Ok(session_blocks.contains(tool_call))
        } else {
            Ok(false)
        }
    }
}

pub struct RedisBlocklist {
    client: redis::Client,
}

impl RedisBlocklist {
    pub fn new(redis_url: &str) -> Result<Self, String> {
        let client = redis::Client::open(redis_url).map_err(|e| e.to_string())?;
        Ok(Self { client })
    }
}

#[async_trait]
impl BlocklistStore for RedisBlocklist {
    async fn add_block(&self, session_id: &str, tool_call: &str) -> Result<(), String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;
        
        let key = format!("microloop:blocklist:{}", session_id);
        let _: () = con.sadd(key, tool_call).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn is_blocked(&self, session_id: &str, tool_call: &str) -> Result<bool, String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;
            
        let key = format!("microloop:blocklist:{}", session_id);
        let is_member: bool = con.sismember(key, tool_call).await.map_err(|e| e.to_string())?;
        Ok(is_member)
    }
}

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

type ValueCell = Arc<RwLock<Vec<u8>>>;

struct Shard {
    map: RwLock<HashMap<String, ValueCell>>,
}

pub struct ShardedDb {
    shards: Vec<Arc<Shard>>,
}

impl ShardedDb {
    pub fn new(shard_count: usize) -> Self {
        assert!(shard_count > 0, "shard_count must be greater than 0");

        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(Arc::new(Shard {
                map: RwLock::new(HashMap::new()),
            }));
        }
        Self { shards }
    }

    fn shard_for(&self, key: &str) -> Arc<Shard> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let idx = (hasher.finish() as usize) % self.shards.len();
        self.shards[idx].clone()
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let shard = self.shard_for(key);
        let cell = {
            let guard = shard.map.read().await;
            guard.get(key).cloned()
        };

        match cell {
            Some(cell) => {
                let value_guard = cell.read().await;
                Some(value_guard.clone())
            }
            None => None,
        }
    }

    pub async fn put(&self, key: String, value: Vec<u8>) {
        let shard = self.shard_for(&key);
        let existing_cell = {
            let guard = shard.map.read().await;
            guard.get(&key).cloned()
        };

        if let Some(cell) = existing_cell {
            let mut value_guard = cell.write().await;
            *value_guard = value;
            return;
        }

        let mut guard = shard.map.write().await;
        guard.insert(key, Arc::new(RwLock::new(value)));
    }

    pub async fn delete(&self, key: &str) -> bool {
        let shard = self.shard_for(key);
        let mut guard = shard.map.write().await;
        guard.remove(key).is_some()
    }
}
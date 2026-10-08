use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use async_lock::RwLock;

const NUM_SHARDS: usize = 16;

pub struct ShardMap<K, V> {
    shards: [RwLock<HashMap<K, V>>; NUM_SHARDS],
}

#[allow(clippy::future_not_send, reason = "Futures are Send if type args are")]
impl<K: Hash + Eq, V> ShardMap<K, V> {
    pub fn new() -> Self {
        let shards = std::array::from_fn(|_| RwLock::default());
        Self { shards }
    }

    fn get_shard_index(&self, shard_key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        shard_key.hash(&mut hasher);
        #[allow(clippy::cast_possible_truncation, reason = "Truncation won't affect the result of the modulo operation")]
        { hasher.finish() as usize % self.shards.len() }
    }

    /// Inserts the given value to the map, keyed by the given key
    /// returns the existing value at the given key if present
    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let idx = self.get_shard_index(&key);
        let mut shard = self.shards[idx].write().await;
        shard.insert(key, value)
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let idx = self.get_shard_index(key);
        let mut shard = self.shards[idx].write().await;
        shard.remove(key)
    }
}
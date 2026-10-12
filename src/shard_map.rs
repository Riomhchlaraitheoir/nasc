use async_lock::Mutex;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

const NUM_SHARDS: usize = 16;

#[derive(Debug)]
pub struct ShardMap<K, V> {
    shards: [Mutex<HashMap<K, V>>; NUM_SHARDS],
}

impl<K, V> Default for ShardMap<K, V> {
    fn default() -> Self {
        let shards = std::array::from_fn(|_| Mutex::default());
        Self { shards }
    }
}

#[allow(clippy::future_not_send, reason = "Futures are Send if type args are")]
impl<K: Hash + Eq, V> ShardMap<K, V> {
    pub fn new() -> Self {
        Self::default()
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
        let mut shard = self.shards[idx].lock().await;
        shard.insert(key, value)
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let idx = self.get_shard_index(key);
        let mut shard = self.shards[idx].lock().await;
        shard.remove(key)
    }
}
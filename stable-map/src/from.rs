#[cfg(test)]
mod tests;

use {
    crate::StableMap,
    core::hash::{BuildHasher, Hash},
    hashbrown::HashMap,
};

impl<K, V, S, const N: usize> From<[(K, V); N]> for StableMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn from(value: [(K, V); N]) -> Self {
        let mut map = Self::with_capacity_and_hasher(N, S::default());
        for (k, v) in value {
            map.insert(k, v);
        }
        map
    }
}

impl<K, V, S> From<HashMap<K, V, S>> for StableMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Clone,
{
    fn from(value: HashMap<K, V, S>) -> Self {
        let mut map = Self::with_capacity_and_hasher(value.len(), value.hasher().clone());
        for (k, v) in value {
            map.insert(k, v);
        }
        map
    }
}

impl<K, V, S> From<StableMap<K, V, S>> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Clone,
{
    fn from(value: StableMap<K, V, S>) -> Self {
        let mut map = Self::with_capacity_and_hasher(value.len(), value.hasher().clone());
        for (k, v) in value {
            map.insert(k, v);
        }
        map
    }
}

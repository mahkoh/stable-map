#[cfg(test)]
mod tests;

use {
    crate::StableMap,
    core::hash::{BuildHasher, Hash},
};

impl<K, V, S> Clone for StableMap<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: BuildHasher + Clone,
{
    fn clone(&self) -> Self {
        let mut map = Self::with_capacity_and_hasher(self.len(), self.hasher().clone());
        for (k, v) in self {
            unsafe {
                // SAFETY:
                // - All k are part of the same hash map so they must be distinct.
                map.insert_unique_unchecked(k.clone(), v.clone());
            }
        }
        map
    }
}

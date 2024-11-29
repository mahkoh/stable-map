#[cfg(test)]
mod tests;

use {
    crate::StableMap,
    core::hash::{BuildHasher, Hash},
};

impl<K, V, S> Eq for StableMap<K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
}

impl<K, V, S> PartialEq for StableMap<K, V, S>
where
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (k, v) in self {
            if other.get(k) != Some(v) {
                return false;
            }
        }
        true
    }
}

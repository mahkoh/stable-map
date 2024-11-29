#[cfg(test)]
mod tests;

use {
    crate::StableMap,
    core::hash::{BuildHasher, Hash},
};

impl<K, V, S> FromIterator<(K, V)> for StableMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::default();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

#[cfg(test)]
mod tests;

use {
    crate::StableMap,
    core::hash::{BuildHasher, Hash},
};

impl<'a, K, V, S> Extend<&'a (K, V)> for StableMap<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = &'a (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k.clone(), v.clone());
        }
    }
}

impl<'a, K, V, S> Extend<(&'a K, &'a V)> for StableMap<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k.clone(), v.clone());
        }
    }
}

impl<K, V, S> Extend<(K, V)> for StableMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

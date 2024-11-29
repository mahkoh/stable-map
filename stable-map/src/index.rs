#[cfg(test)]
mod tests;

use {
    crate::StableMap,
    core::{
        hash::{BuildHasher, Hash},
        ops::Index,
    },
    hashbrown::Equivalent,
};

impl<K, Q, V, S> Index<&Q> for StableMap<K, V, S>
where
    K: Eq + Hash,
    Q: Hash + Equivalent<K> + ?Sized,
    S: BuildHasher,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

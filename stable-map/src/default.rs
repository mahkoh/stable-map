use crate::map::StableMap;

impl<K, V, S> Default for StableMap<K, V, S>
where
    S: Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

use {
    crate::StableMap,
    core::fmt::{Debug, Formatter},
};

impl<K, V, S> Debug for StableMap<K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut map = f.debug_map();
        for (k, v) in self {
            map.entry(k, v);
        }
        map.finish()
    }
}

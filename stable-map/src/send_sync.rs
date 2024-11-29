use crate::StableMap;

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V, S> Send for StableMap<K, V, S>
where
    K: Send,
    V: Send,
    S: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V, S> Sync for StableMap<K, V, S>
where
    K: Sync,
    V: Sync,
    S: Sync,
{
}

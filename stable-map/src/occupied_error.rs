use {
    crate::entry::OccupiedEntry,
    core::fmt::{Debug, Display, Formatter},
};

/// The error returned by [`try_insert`](crate::StableMap::try_insert) when the key already exists.
///
/// Contains the occupied entry, and the value that was not inserted.
///
/// # Examples
///
/// ```
/// use stable_map::{OccupiedError, StableMap};
///
/// let mut map: StableMap<_, _> = [("a", 10), ("b", 20)].into();
///
/// // try_insert method returns mutable reference to the value if keys are vacant,
/// // but if the map did have key present, nothing is updated, and the provided
/// // value is returned inside `Err(_)` variant
/// match map.try_insert("a", 100) {
///     Err(OccupiedError { mut entry, value }) => {
///         assert_eq!(entry.key(), &"a");
///         assert_eq!(value, 100);
///         assert_eq!(entry.insert(100), 10)
///     }
///     _ => unreachable!(),
/// }
/// assert_eq!(map[&"a"], 100);
/// ```
pub struct OccupiedError<'a, K, V, S> {
    /// The entry in the map that was already occupied.
    pub entry: OccupiedEntry<'a, K, V, S>,
    /// The value which was not inserted, because the entry was already occupied.
    pub value: V,
}

impl<K, V, S> Debug for OccupiedError<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OccupiedError")
            .field("key", self.entry.key())
            .field("old_value", self.entry.get())
            .field("new_value", &self.value)
            .finish()
    }
}

impl<K, V, S> Display for OccupiedError<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "failed to insert {:?}, key {:?} already exists with value {:?}",
            self.value,
            self.entry.key(),
            self.entry.get(),
        )
    }
}

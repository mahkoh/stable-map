#[cfg(test)]
mod tests;

use {
    crate::pos_vec::{
        pos::{InUse, Pos},
        PosVecRawAccess,
    },
    core::{
        fmt::{Debug, Formatter},
        iter::FusedIterator,
    },
    hashbrown::hash_map,
};

/// A mutable iterator over the values of a `StableMap` in arbitrary order.
/// The iterator element type is `&'a mut V`.
///
/// This `struct` is created by the [`values_mut`] method on [`StableMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: crate::StableMap::values_mut
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let mut map: StableMap<_, _> = [(1, "One".to_owned()), (2, "Two".into())].into();
///
/// let mut values = map.values_mut();
/// values.next().map(|v| v.push_str(" Mississippi"));
/// values.next().map(|v| v.push_str(" Mississippi"));
///
/// // It is fused iterator
/// assert_eq!(values.next(), None);
/// assert_eq!(values.next(), None);
///
/// assert_eq!(map.get(&1).unwrap(), &"One Mississippi".to_owned());
/// assert_eq!(map.get(&2).unwrap(), &"Two Mississippi".to_owned());
/// ```
pub struct ValuesMut<'a, K, V> {
    pub(crate) iter: hash_map::ValuesMut<'a, K, Pos<InUse>>,
    pub(crate) storage: PosVecRawAccess<'a, V>,
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.iter.next()?;
        let value = unsafe { self.storage.get_unchecked_mut(pos) };
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> Debug for ValuesMut<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ValuesMut").finish_non_exhaustive()
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Send for ValuesMut<'_, K, V>
where
    K: Send,
    V: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Sync for ValuesMut<'_, K, V>
where
    K: Sync,
    V: Sync,
{
}

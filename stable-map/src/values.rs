#[cfg(test)]
mod tests;

use {
    crate::{
        linear_storage::LinearStorage,
        pos_vec::pos::{InUse, Pos},
    },
    core::{
        fmt::{Debug, Formatter},
        iter::FusedIterator,
    },
    hashbrown::hash_map,
};

/// An iterator over the values of a `StableMap` in arbitrary order.
/// The iterator element type is `&'a V`.
///
/// This `struct` is created by the [`values`] method on [`StableMap`]. See its
/// documentation for more.
///
/// [`values`]: crate::StableMap::values
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let map: StableMap<_, _> = [(1, "a"), (2, "b"), (3, "c")].into();
///
/// let mut values = map.values();
/// let mut vec = vec![values.next(), values.next(), values.next()];
///
/// // The `Values` iterator produces values in arbitrary order, so the
/// // values must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [Some(&"a"), Some(&"b"), Some(&"c")]);
///
/// // It is fused iterator
/// assert_eq!(values.next(), None);
/// assert_eq!(values.next(), None);
/// ```
pub struct Values<'a, K, V> {
    pub(crate) iter: hash_map::Values<'a, K, Pos<InUse>>,
    pub(crate) storage: &'a LinearStorage<V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.iter.next()?;
        let value = unsafe { self.storage.get_unchecked(pos) };
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> Clone for Values<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            storage: self.storage,
        }
    }
}

impl<K, V> Debug for Values<'_, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Send for Values<'_, K, V>
where
    K: Send,
    V: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Sync for Values<'_, K, V>
where
    K: Sync,
    V: Sync,
{
}

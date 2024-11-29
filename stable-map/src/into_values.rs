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

/// An owning iterator over the values of a `StableMap` in arbitrary order.
/// The iterator element type is `V`.
///
/// This `struct` is created by the [`into_values`] method on [`StableMap`].
/// See its documentation for more. The map cannot be used after calling that method.
///
/// [`into_values`]: crate::StableMap::into_values
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let map: StableMap<_, _> = [(1, "a"), (2, "b"), (3, "c")].into();
///
/// let mut values = map.into_values();
/// let mut vec = vec![values.next(), values.next(), values.next()];
///
/// // The `IntoValues` iterator produces values in arbitrary order, so
/// // the values must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [Some("a"), Some("b"), Some("c")]);
///
/// // It is fused iterator
/// assert_eq!(values.next(), None);
/// assert_eq!(values.next(), None);
/// ```
pub struct IntoValues<K, V> {
    pub(crate) iter: hash_map::IntoValues<K, Pos<InUse>>,
    pub(crate) storage: LinearStorage<V>,
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.iter.next()?;
        let value = unsafe { self.storage.take_unchecked(pos) };
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> Debug for IntoValues<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IntoValues").finish_non_exhaustive()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> Default for IntoValues<K, V> {
    fn default() -> Self {
        Self {
            iter: Default::default(),
            storage: LinearStorage::with_capacity(0),
        }
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Send for IntoValues<K, V>
where
    K: Send,
    V: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Sync for IntoValues<K, V>
where
    K: Sync,
    V: Sync,
{
}

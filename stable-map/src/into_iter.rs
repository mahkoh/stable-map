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

/// An owning iterator over the entries of a `StableMap` in arbitrary order.
/// The iterator element type is `(K, V)`.
///
/// This `struct` is created by the [`into_iter`] method on [`StableMap`]
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
/// The map cannot be used after calling that method.
///
/// [`into_iter`]: crate::StableMap::into_iter
/// [`StableMap`]: crate::StableMap
/// [`IntoIterator`]: https://doc.rust-lang.org/core/iter/trait.IntoIterator.html
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let map: StableMap<_, _> = [(1, "a"), (2, "b"), (3, "c")].into();
///
/// let mut iter = map.into_iter();
/// let mut vec = vec![iter.next(), iter.next(), iter.next()];
///
/// // The `IntoIter` iterator produces items in arbitrary order, so the
/// // items must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [Some((1, "a")), Some((2, "b")), Some((3, "c"))]);
///
/// // It is fused iterator
/// assert_eq!(iter.next(), None);
/// assert_eq!(iter.next(), None);
/// ```
pub struct IntoIter<K, V> {
    pub(crate) iter: hash_map::IntoIter<K, Pos<InUse>>,
    pub(crate) storage: LinearStorage<V>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, pos) = self.iter.next()?;
        let v = unsafe {
            // SAFETY:
            // - By the invariants, pos is valid.
            self.storage.take_unchecked(pos)
        };
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IntoIter").finish_non_exhaustive()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> Default for IntoIter<K, V> {
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
unsafe impl<K, V> Send for IntoIter<K, V>
where
    K: Send,
    V: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Sync for IntoIter<K, V>
where
    K: Sync,
    V: Sync,
{
}

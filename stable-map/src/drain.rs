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

/// A draining iterator over the entries of a `StableMap` in arbitrary
/// order. The iterator element type is `(K, V)`.
///
/// This `struct` is created by the [`drain`] method on [`StableMap`]. See its
/// documentation for more.
///
/// [`drain`]: crate::StableMap::drain
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let mut map: StableMap<_, _> = [(1, "a"), (2, "b"), (3, "c")].into();
///
/// let mut drain_iter = map.drain();
/// let mut vec = vec![drain_iter.next(), drain_iter.next(), drain_iter.next()];
///
/// // The `Drain` iterator produces items in arbitrary order, so the
/// // items must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [Some((1, "a")), Some((2, "b")), Some((3, "c"))]);
///
/// // It is fused iterator
/// assert_eq!(drain_iter.next(), None);
/// assert_eq!(drain_iter.next(), None);
/// ```
pub struct Drain<'a, K, V> {
    pub(crate) drain: hash_map::Drain<'a, K, Pos<InUse>>,
    pub(crate) entries: &'a mut LinearStorage<V>,
}

impl<K, V> Drop for Drain<'_, K, V> {
    fn drop(&mut self) {
        self.entries.clear();
        // SAFETY(invariants):
        // - Dropping hash_map::Drain clears key_to_pos.
    }
}

impl<K, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, pos) = self.drain.next()?;
        let value = unsafe {
            // SAFETY: By the invariants, pos is valid.
            self.entries.take_unchecked(pos)
        };
        Some((k, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<K, V> Debug for Drain<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Drain").finish_non_exhaustive()
    }
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V> {
    fn len(&self) -> usize {
        self.drain.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> {}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Send for Drain<'_, K, V>
where
    K: Send,
    V: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Sync for Drain<'_, K, V>
where
    K: Sync,
    V: Sync,
{
}

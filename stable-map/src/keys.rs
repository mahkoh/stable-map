#[cfg(test)]
mod tests;

use {
    crate::pos_vec::pos::{InUse, Pos},
    core::{
        fmt::{Debug, Formatter},
        iter::FusedIterator,
    },
    hashbrown::hash_map,
};

/// An iterator over the keys of a `StableMap` in arbitrary order.
/// The iterator element type is `&'a K`.
///
/// This `struct` is created by the [`keys`] method on [`StableMap`]. See its
/// documentation for more.
///
/// [`keys`]: crate::StableMap::keys
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let map: StableMap<_, _> = [(1, "a"), (2, "b"), (3, "c")].into();
///
/// let mut keys = map.keys();
/// let mut vec = vec![keys.next(), keys.next(), keys.next()];
///
/// // The `Keys` iterator produces keys in arbitrary order, so the
/// // keys must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [Some(&1), Some(&2), Some(&3)]);
///
/// // It is fused iterator
/// assert_eq!(keys.next(), None);
/// assert_eq!(keys.next(), None);
/// ```
pub struct Keys<'a, K> {
    pub(crate) iter: hash_map::Keys<'a, K, Pos<InUse>>,
}

impl<'a, K> Iterator for Keys<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K> Clone for Keys<'_, K> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<K> Debug for Keys<'_, K>
where
    K: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K> FusedIterator for Keys<'_, K> {}

impl<K> ExactSizeIterator for Keys<'_, K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K> Send for Keys<'_, K> where K: Send {}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K> Sync for Keys<'_, K> where K: Sync {}

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

/// An owning iterator over the keys of a `StableMap` in arbitrary order.
/// The iterator element type is `K`.
///
/// This `struct` is created by the [`into_keys`] method on [`StableMap`].
/// See its documentation for more.
/// The map cannot be used after calling that method.
///
/// [`into_keys`]: crate::StableMap::into_keys
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let map: StableMap<_, _> = [(1, "a"), (2, "b"), (3, "c")].into();
///
/// let mut keys = map.into_keys();
/// let mut vec = vec![keys.next(), keys.next(), keys.next()];
///
/// // The `IntoKeys` iterator produces keys in arbitrary order, so the
/// // keys must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [Some(1), Some(2), Some(3)]);
///
/// // It is fused iterator
/// assert_eq!(keys.next(), None);
/// assert_eq!(keys.next(), None);
/// ```
pub struct IntoKeys<K> {
    pub(crate) iter: hash_map::IntoKeys<K, Pos<InUse>>,
}

impl<K> Iterator for IntoKeys<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K> Debug for IntoKeys<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IntoKeys").finish_non_exhaustive()
    }
}

impl<K> FusedIterator for IntoKeys<K> {}

impl<K> ExactSizeIterator for IntoKeys<K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K> Default for IntoKeys<K> {
    fn default() -> Self {
        Self {
            iter: Default::default(),
        }
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K> Send for IntoKeys<K> where K: Send {}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K> Sync for IntoKeys<K> where K: Sync {}

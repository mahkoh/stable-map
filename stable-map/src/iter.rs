#[cfg(test)]
mod tests;

use {
    crate::{
        linear_storage::LinearStorage,
        map::StableMap,
        pos_vec::pos::{InUse, Pos},
    },
    core::{
        fmt::{Debug, Formatter},
        iter::FusedIterator,
    },
    hashbrown::hash_map,
};

/// An iterator over the entries of a `StableMap` in arbitrary order.
/// The iterator element type is `(&'a K, &'a V)`.
///
/// This `struct` is created by the [`iter`] method on [`StableMap`]. See its
/// documentation for more.
///
/// [`iter`]: crate::StableMap
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let map: StableMap<_, _> = [(1, "a"), (2, "b"), (3, "c")].into();
///
/// let mut iter = map.iter();
/// let mut vec = vec![iter.next(), iter.next(), iter.next()];
///
/// // The `Iter` iterator produces items in arbitrary order, so the
/// // items must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [Some((&1, &"a")), Some((&2, &"b")), Some((&3, &"c"))]);
///
/// // It is fused iterator
/// assert_eq!(iter.next(), None);
/// assert_eq!(iter.next(), None);
/// ```
pub struct Iter<'a, K, V> {
    pub(crate) iter: hash_map::Iter<'a, K, Pos<InUse>>,
    pub(crate) entries: &'a LinearStorage<V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, pos) = self.iter.next()?;
        let v = unsafe { self.entries.get_unchecked(pos) };
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V, S> IntoIterator for &'a StableMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            entries: self.entries,
        }
    }
}

impl<K, V> Debug for Iter<'_, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Send for Iter<'_, K, V>
where
    K: Send,
    V: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Sync for Iter<'_, K, V>
where
    K: Sync,
    V: Sync,
{
}

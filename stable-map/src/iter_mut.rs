#[cfg(test)]
mod tests;

use {
    crate::{
        map::StableMap,
        pos_vec::{
            pos::{InUse, Pos},
            PosVecRawAccess,
        },
    },
    core::{
        fmt::{Debug, Formatter},
        iter::FusedIterator,
    },
    hashbrown::hash_map,
};

/// A mutable iterator over the entries of a `StableMap` in arbitrary order.
/// The iterator element type is `(&'a K, &'a mut V)`.
///
/// This `struct` is created by the [`iter_mut`] method on [`StableMap`]. See its
/// documentation for more.
///
/// [`iter_mut`]: crate::StableMap::iter_mut
/// [`StableMap`]: crate::StableMap
///
/// # Examples
///
/// ```
/// use stable_map::StableMap;
///
/// let mut map: StableMap<_, _> = [(1, "One".to_owned()), (2, "Two".into())].into();
///
/// let mut iter = map.iter_mut();
/// iter.next().map(|(_, v)| v.push_str(" Mississippi"));
/// iter.next().map(|(_, v)| v.push_str(" Mississippi"));
///
/// // It is fused iterator
/// assert_eq!(iter.next(), None);
/// assert_eq!(iter.next(), None);
///
/// assert_eq!(map.get(&1).unwrap(), &"One Mississippi".to_owned());
/// assert_eq!(map.get(&2).unwrap(), &"Two Mississippi".to_owned());
/// ```
pub struct IterMut<'a, K, V> {
    pub(crate) iter: hash_map::IterMut<'a, K, Pos<InUse>>,
    pub(crate) entries: PosVecRawAccess<'a, V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, pos) = self.iter.next()?;
        let value = unsafe { self.entries.get_unchecked_mut(pos) };
        Some((k, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut StableMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> Debug for IterMut<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IterMut").finish_non_exhaustive()
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Send for IterMut<'_, K, V>
where
    K: Send,
    V: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V> Sync for IterMut<'_, K, V>
where
    K: Sync,
    V: Sync,
{
}

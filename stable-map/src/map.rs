#[cfg(test)]
mod tests;

use {
    crate::{
        drain::Drain,
        entry::{Entry, EntryRef, OccupiedEntry, VacantEntry, VacantEntryRef},
        into_iter::IntoIter,
        into_keys::IntoKeys,
        into_values::IntoValues,
        iter::Iter,
        iter_mut::IterMut,
        keys::Keys,
        linear_storage::LinearStorage,
        occupied_error::OccupiedError,
        pos_vec::pos::{InUse, Pos},
        values::Values,
        values_mut::ValuesMut,
    },
    core::{
        cmp::min,
        hash::{BuildHasher, Hash},
        iter::FusedIterator,
        marker::PhantomData,
        mem::{self},
    },
    hashbrown::{hash_map, DefaultHashBuilder, Equivalent, HashMap},
};

/// A hash map with temporarily-stable indices.
///
/// This is a small wrapper around a [HashMap<K, V>] that splits the map into two parts:
///
/// - `HashMap<K, usize>`
/// - `Vec<V>`
///
/// The index of for each key stays the same unless the key is removed from the map or the
/// map is explicitly compacted.
///
/// # Example
///
/// Consider a service that allows clients to register callbacks:
///
/// ```
/// use {
///     parking_lot::Mutex,
///     stable_map::StableMap,
///     std::sync::{
///         atomic::{AtomicUsize, Ordering::Relaxed},
///         Arc,
///     },
/// };
///
/// pub struct Service {
///     next_callback_id: AtomicUsize,
///     callbacks: Mutex<StableMap<CallbackId, Arc<dyn Callback>>>,
/// }
///
/// pub trait Callback {
///     fn run(&self);
/// }
///
/// #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
/// pub struct CallbackId(usize);
///
/// impl Service {
///     pub fn register_callback(&self, callback: Arc<dyn Callback>) -> CallbackId {
///         let id = CallbackId(self.next_callback_id.fetch_add(1, Relaxed));
///         self.callbacks.lock().insert(id, callback);
///         id
///     }
///
///     pub fn unregister_callback(&self, id: CallbackId) {
///         self.callbacks.lock().remove(&id);
///     }
///
///     fn execute_callbacks(&self) {
///         let mut callbacks = self.callbacks.lock();
///         for i in 0..callbacks.index_len() {
///             if let Some(callback) = callbacks.get_by_index(i).cloned() {
///                 // Drop the mutex so that the callback can itself call
///                 // register_callback or unregister_callback.
///                 drop(callbacks);
///                 // Run the callback.
///                 callback.run();
///                 // Re-acquire the mutex.
///                 callbacks = self.callbacks.lock();
///             }
///         }
///         // Compact the map so that index_len does not grow much larger than the actual
///         // size of the map.
///         callbacks.compact();
///     }
/// }
/// ```
//
// This type upholds the following invariants:
//
// - key_to_pos contains only valid Pos<InUse> returned by storage.
//
// SAFETY:
// - LinearStorage::clear invalidates existing Pos<InUse> without consuming them.
// - Code calling LinearStorage::clear must explain how it upholds the invariant.
pub struct StableMap<K, V, S = DefaultHashBuilder> {
    key_to_pos: HashMap<K, Pos<InUse>, S>,
    storage: LinearStorage<V>,
}

#[cfg(feature = "default-hasher")]
impl<K, V> StableMap<K, V, DefaultHashBuilder> {
    /// Creates an empty `StableMap`.
    ///
    /// The map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    /// let mut map: StableMap<&str, i32> = StableMap::new();
    /// assert_eq!(map.len(), 0);
    /// assert_eq!(map.capacity(), 0);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new() -> Self {
        Self {
            key_to_pos: HashMap::new(),
            storage: LinearStorage::with_capacity(0),
        }
    }

    /// Creates an empty `StableMap` with the specified capacity.
    ///
    /// The map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    /// let mut map: StableMap<&str, i32> = StableMap::with_capacity(10);
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            key_to_pos: HashMap::with_capacity(capacity),
            storage: LinearStorage::with_capacity(capacity),
        }
    }
}

impl<K, V, S> StableMap<K, V, S> {
    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the `StableMap<K, V>` might be able to hold
    /// more, but is guaranteed to be able to hold at least this many.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    /// let map: StableMap<i32, i32> = StableMap::with_capacity(100);
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 100);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn capacity(&self) -> usize {
        min(self.key_to_pos.capacity(), self.storage.capacity())
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// a.insert(1, "a");
    /// let capacity_before_clear = a.capacity();
    ///
    /// a.clear();
    ///
    /// // Map is empty.
    /// assert!(a.is_empty());
    /// // But map capacity is equal to old one.
    /// assert_eq!(a.capacity(), capacity_before_clear);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn clear(&mut self) {
        self.key_to_pos.clear();
        self.storage.clear();
        // SAFETY(invariants):
        // - We have cleared key_to_pos.
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        self.key_to_pos.contains_key(key)
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the
    /// allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed, it
    /// drops the remaining key-value pairs. The returned iterator keeps a
    /// mutable borrow on the vector to optimize its implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// let capacity_before_drain = a.capacity();
    ///
    /// for (k, v) in a.drain().take(1) {
    ///     assert!(k == 1 || k == 2);
    ///     assert!(v == "a" || v == "b");
    /// }
    ///
    /// // As we can see, the map is empty and contains no element.
    /// assert!(a.is_empty() && a.len() == 0);
    /// // But map capacity is equal to old one.
    /// assert_eq!(a.capacity(), capacity_before_drain);
    ///
    /// let mut a = StableMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    ///
    /// {   // Iterator is dropped without being consumed.
    ///     let d = a.drain();
    /// }
    ///
    /// // But the map is empty even if we do not use Drain iterator.
    /// assert!(a.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        Drain {
            drain: self.key_to_pos.drain(),
            entries: &mut self.storage,
        }
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut letters = StableMap::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     let counter = letters.entry(ch).or_insert(0);
    ///     *counter += 1;
    /// }
    ///
    /// assert_eq!(letters[&'s'], 2);
    /// assert_eq!(letters[&'t'], 3);
    /// assert_eq!(letters[&'u'], 1);
    /// assert_eq!(letters.get(&'y'), None);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, S>
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        match self.key_to_pos.entry(key) {
            hash_map::Entry::Occupied(v) => Entry::Occupied(OccupiedEntry {
                entry: v,
                entries: &mut self.storage,
            }),
            hash_map::Entry::Vacant(v) => Entry::Vacant(VacantEntry {
                entry: v,
                entries: &mut self.storage,
            }),
        }
    }

    /// Gets the given key's corresponding entry by reference in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut words: StableMap<String, usize> = StableMap::new();
    /// let source = ["poneyland", "horseyland", "poneyland", "poneyland"];
    /// for (i, &s) in source.iter().enumerate() {
    ///     let counter = words.entry_ref(s).or_insert(0);
    ///     *counter += 1;
    /// }
    ///
    /// assert_eq!(words["poneyland"], 3);
    /// assert_eq!(words["horseyland"], 1);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn entry_ref<'b, Q>(&mut self, key: &'b Q) -> EntryRef<'_, 'b, K, Q, V, S>
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        match self.key_to_pos.entry_ref(key) {
            hash_map::EntryRef::Occupied(v) => EntryRef::Occupied(OccupiedEntry {
                entry: v,
                entries: &mut self.storage,
            }),
            hash_map::EntryRef::Vacant(v) => EntryRef::Vacant(VacantEntryRef {
                entry: v,
                entries: &mut self.storage,
            }),
        }
    }

    /// Drains elements which are true under the given predicate,
    /// and returns an iterator over the removed items.
    ///
    /// In other words, move all pairs `(k, v)` such that `f(&k, &mut v)` returns `true` out
    /// into another iterator.
    ///
    /// Note that `extract_if` lets you mutate every value in the filter closure, regardless of
    /// whether you choose to keep or remove it.
    ///
    /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating
    /// or the iteration short-circuits, then the remaining elements will be retained.
    /// Use [`retain()`] with a negated predicate if you do not need the returned iterator.
    ///
    /// Keeps the allocated memory for reuse.
    ///
    /// [`retain()`]: StableMap::retain
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<i32, i32> = (0..8).map(|x| (x, x)).collect();
    ///
    /// let drained: StableMap<i32, i32> = map.extract_if(|k, _v| k % 2 == 0).collect();
    ///
    /// let mut evens = drained.keys().cloned().collect::<Vec<_>>();
    /// let mut odds = map.keys().cloned().collect::<Vec<_>>();
    /// evens.sort();
    /// odds.sort();
    ///
    /// assert_eq!(evens, vec![0, 2, 4, 6]);
    /// assert_eq!(odds, vec![1, 3, 5, 7]);
    ///
    /// let mut map: StableMap<i32, i32> = (0..8).map(|x| (x, x)).collect();
    ///
    /// {   // Iterator is dropped without being consumed.
    ///     let d = map.extract_if(|k, _v| k % 2 != 0);
    /// }
    ///
    /// // ExtractIf was not exhausted, therefore no elements were drained.
    /// assert_eq!(map.len(), 8);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn extract_if<F>(
        &mut self,
        mut f: F,
    ) -> impl FusedIterator<Item = (K, V)> + use<'_, K, V, F, S>
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        // SAFETY: (applies to all dereferences of storage below)
        // - storage points to self.storage which remains valid since the
        //   return value borrows self
        // - all references to self.storage by the return value are created through
        //   this pointer, therefore it is sufficient to show that we don't create more
        //   than one reference at a time.
        // - the first dereference is live only for the lifetime of the particular closure
        //   invocation. this is a FnMut closure, therefore it cannot run concurrently
        //   with itself.
        // - the second dereference is live only during the next method call and strictly
        //   after the nested next call.
        // - the first dereference is only invoked through the nested next call.
        // - the user-defined callback cannot invoke the outer next function since that
        //   would create multiple multiple references to the iterator.
        let storage = &raw mut self.storage;
        let iter = self.key_to_pos.extract_if(move |k, pos| {
            let storage = unsafe {
                // SAFETY: see comment at the top
                &mut *storage
            };
            let v = unsafe {
                // SAFETY: By the invariants, pos is valid
                storage.get_unchecked_mut(pos)
            };
            f(k, v)
        });
        struct Iter<'a, K, V, I> {
            iter: I,
            storage: *mut LinearStorage<V>,
            _phantom1: PhantomData<fn() -> K>,
            _phantom2: PhantomData<&'a mut LinearStorage<V>>,
        }
        impl<K, V, I> Iterator for Iter<'_, K, V, I>
        where
            I: Iterator<Item = (K, Pos<InUse>)>,
        {
            type Item = (K, V);

            fn next(&mut self) -> Option<Self::Item> {
                let (k, pos) = self.iter.next()?;
                let storage = unsafe {
                    // SAFETY: see comment at the top
                    &mut *self.storage
                };
                let value = unsafe {
                    // SAFETY: By the invariants, pos is valid
                    storage.take_unchecked(pos)
                };
                Some((k, value))
            }
        }
        impl<K, V, I> FusedIterator for Iter<'_, K, V, I> where I: FusedIterator<Item = (K, Pos<InUse>)> {}
        Iter::<'_, K, V, _> {
            iter,
            storage,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let pos = self.key_to_pos.get(key)?;
        let v = unsafe {
            // SAFETY:
            // - By the invariants, pos is valid
            self.storage.get_unchecked(pos)
        };
        Some(v)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    #[inline]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let (k, pos) = self.key_to_pos.get_key_value(key)?;
        let v = unsafe {
            // SAFETY:
            // - By the invariants, pos is valid
            self.storage.get_unchecked(pos)
        };
        Some((k, v))
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    #[inline]
    pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let (k, pos) = self.key_to_pos.get_key_value(key)?;
        let value = unsafe {
            // SAFETY:
            // - By the invariants, pos is valid
            self.storage.get_unchecked_mut(pos)
        };
        Some((k, value))
    }

    /// Attempts to get mutable references to `N` values in the map at once, with immutable
    /// references to the corresponding keys.
    ///
    /// Returns an array of length `N` with the results of each query. For soundness, at most one
    /// mutable reference will be returned to any value. `None` will be used if the key is missing.
    ///
    /// # Panics
    ///
    /// Panics if any keys are overlapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut libraries = StableMap::new();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// let got = libraries.get_many_key_value_mut([
    ///     "Bodleian Library",
    ///     "Herzogin-Anna-Amalia-Bibliothek",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some((&"Bodleian Library".to_string(), &mut 1602)),
    ///         Some((&"Herzogin-Anna-Amalia-Bibliothek".to_string(), &mut 1691)),
    ///     ],
    /// );
    /// // Missing keys result in None
    /// let got = libraries.get_many_key_value_mut([
    ///     "Bodleian Library",
    ///     "Gewandhaus",
    /// ]);
    /// assert_eq!(got, [Some((&"Bodleian Library".to_string(), &mut 1602)), None]);
    /// ```
    ///
    /// ```should_panic
    /// use stable_map::StableMap;
    ///
    /// let mut libraries = StableMap::new();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    ///
    /// // Duplicate keys result in panic!
    /// let got = libraries.get_many_key_value_mut([
    ///     "Bodleian Library",
    ///     "Herzogin-Anna-Amalia-Bibliothek",
    ///     "Herzogin-Anna-Amalia-Bibliothek",
    /// ]);
    /// ```
    pub fn get_many_key_value_mut<Q, const N: usize>(
        &mut self,
        ks: [&Q; N],
    ) -> [Option<(&K, &mut V)>; N]
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let ps = self.key_to_pos.get_many_key_value_mut::<Q, N>(ks);
        unsafe {
            // SAFETY:
            // - By the invariants, all pos are valid
            self.storage
                .get_many_unchecked_mut(ps, |p| p.1, |(k, _), v| (k, v))
        }
    }

    /// Attempts to get mutable references to `N` values in the map at once, with immutable
    /// references to the corresponding keys, without validating that the values are unique.
    ///
    /// Returns an array of length `N` with the results of each query. `None` will be returned if
    /// any of the keys are missing.
    ///
    /// For a safe alternative see [`get_many_key_value_mut`](`StableMap::get_many_key_value_mut`).
    ///
    /// # Safety
    ///
    /// Calling this method with overlapping keys is *[undefined behavior]* even if the resulting
    /// references are not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut libraries = StableMap::new();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// let got = libraries.get_many_key_value_mut([
    ///     "Bodleian Library",
    ///     "Herzogin-Anna-Amalia-Bibliothek",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some((&"Bodleian Library".to_string(), &mut 1602)),
    ///         Some((&"Herzogin-Anna-Amalia-Bibliothek".to_string(), &mut 1691)),
    ///     ],
    /// );
    /// // Missing keys result in None
    /// let got = libraries.get_many_key_value_mut([
    ///     "Bodleian Library",
    ///     "Gewandhaus",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some((&"Bodleian Library".to_string(), &mut 1602)),
    ///         None,
    ///     ],
    /// );
    /// ```
    pub unsafe fn get_many_key_value_unchecked_mut<Q, const N: usize>(
        &mut self,
        ks: [&Q; N],
    ) -> [Option<(&K, &mut V)>; N]
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let ps = unsafe {
            // SAFETY: The requirements are forwarded to the caller.
            self.key_to_pos.get_many_key_value_unchecked_mut::<Q, N>(ks)
        };
        unsafe {
            // SAFETY:
            // - By the invariants, all pos are valid
            self.storage
                .get_many_unchecked_mut(ps, |p| p.1, |(k, _), v| (k, v))
        }
    }

    /// Attempts to get mutable references to `N` values in the map at once.
    ///
    /// Returns an array of length `N` with the results of each query. For soundness, at most one
    /// mutable reference will be returned to any value. `None` will be used if the key is missing.
    ///
    /// # Panics
    ///
    /// Panics if any keys are overlapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut libraries = StableMap::new();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// // Get Athenæum and Bodleian Library
    /// let [Some(a), Some(b)] = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "Bodleian Library",
    /// ]) else { panic!() };
    ///
    /// // Assert values of Athenæum and Library of Congress
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "Library of Congress",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some(&mut 1807),
    ///         Some(&mut 1800),
    ///     ],
    /// );
    ///
    /// // Missing keys result in None
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "New York Public Library",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some(&mut 1807),
    ///         None
    ///     ]
    /// );
    /// ```
    ///
    /// ```should_panic
    /// use stable_map::StableMap;
    ///
    /// let mut libraries = StableMap::new();
    /// libraries.insert("Athenæum".to_string(), 1807);
    ///
    /// // Duplicate keys panic!
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "Athenæum",
    /// ]);
    /// ```
    pub fn get_many_mut<Q, const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&mut V>; N]
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let ps = self.key_to_pos.get_many_mut::<Q, N>(ks);
        unsafe {
            // SAFETY:
            // - By the invariants, all pos are valid
            self.storage.get_many_unchecked_mut(ps, |p| p, |_, v| v)
        }
    }

    /// Attempts to get mutable references to `N` values in the map at once, without validating that
    /// the values are unique.
    ///
    /// Returns an array of length `N` with the results of each query. `None` will be used if
    /// the key is missing.
    ///
    /// For a safe alternative see [`get_many_mut`](`StableMap::get_many_mut`).
    ///
    /// # Safety
    ///
    /// Calling this method with overlapping keys is *[undefined behavior]* even if the resulting
    /// references are not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut libraries = StableMap::new();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// // SAFETY: The keys do not overlap.
    /// let [Some(a), Some(b)] = (unsafe { libraries.get_many_unchecked_mut([
    ///     "Athenæum",
    ///     "Bodleian Library",
    /// ]) }) else { panic!() };
    ///
    /// // SAFETY: The keys do not overlap.
    /// let got = unsafe { libraries.get_many_unchecked_mut([
    ///     "Athenæum",
    ///     "Library of Congress",
    /// ]) };
    /// assert_eq!(
    ///     got,
    ///     [
    ///         Some(&mut 1807),
    ///         Some(&mut 1800),
    ///     ],
    /// );
    ///
    /// // SAFETY: The keys do not overlap.
    /// let got = unsafe { libraries.get_many_unchecked_mut([
    ///     "Athenæum",
    ///     "New York Public Library",
    /// ]) };
    /// // Missing keys result in None
    /// assert_eq!(got, [Some(&mut 1807), None]);
    /// ```
    pub unsafe fn get_many_unchecked_mut<Q, const N: usize>(
        &mut self,
        ks: [&Q; N],
    ) -> [Option<&mut V>; N]
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let ps = unsafe {
            // SAFETY: The requirements are forwarded to the caller.
            self.key_to_pos.get_many_unchecked_mut::<Q, N>(ks)
        };
        unsafe {
            // SAFETY:
            // - By the invariants, all pos are valid
            self.storage.get_many_unchecked_mut(ps, |p| p, |_, v| v)
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    ///
    /// assert_eq!(map.get_mut(&2), None);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let pos = self.key_to_pos.get_mut(key)?;
        let value = unsafe {
            // SAFETY:
            // - By the invariants, pos is valid
            self.storage.get_unchecked_mut(pos)
        };
        Some(value)
    }

    /// Returns a reference to the map's [`BuildHasher`].
    ///
    /// [`BuildHasher`]: https://doc.rust-lang.org/std/hash/trait.BuildHasher.html
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::DefaultHashBuilder;
    /// use stable_map::StableMap;
    ///
    /// let hasher = DefaultHashBuilder::default();
    /// let map: StableMap<i32, i32> = StableMap::with_hasher(hasher);
    /// let hasher: &DefaultHashBuilder = map.hasher();
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn hasher(&self) -> &S {
        self.key_to_pos.hasher()
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [`std::collections`]
    /// [module-level documentation] for more.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    /// [`std::collections`]: https://doc.rust-lang.org/std/collections/index.html
    /// [module-level documentation]: https://doc.rust-lang.org/std/collections/index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        match self.key_to_pos.entry(key) {
            hash_map::Entry::Occupied(occupied) => {
                let prev = unsafe {
                    // SAFETY:
                    // - By the invariants, occupied.get() is valid
                    self.storage.get_unchecked_mut(occupied.get())
                };
                Some(mem::replace(prev, value))
            }
            hash_map::Entry::Vacant(vacant) => {
                let pos = self.storage.insert(value);
                vacant.insert(pos);
                None
            }
        }
    }

    /// Insert a key-value pair into the map without checking
    /// if the key already exists in the map.
    ///
    /// This operation is faster than regular insert, because it does not perform
    /// lookup before insertion.
    ///
    /// This operation is useful during initial population of the map.
    /// For example, when constructing a map from another map, we know
    /// that keys are unique.
    ///
    /// Returns a reference to the key and value just inserted.
    ///
    /// # Safety
    ///
    /// This operation is safe if a key does not exist in the map.
    ///
    /// However, if a key exists in the map already, the behavior is unspecified:
    /// this operation may panic, loop forever, or any following operation with the map
    /// may panic, loop forever or return arbitrary result.
    ///
    /// That said, this operation (and following operations) are guaranteed to
    /// not violate memory safety.
    ///
    /// However this operation is still unsafe because the resulting `StableMap`
    /// may be passed to unsafe code which does expect the map to behave
    /// correctly, and would cause unsoundness as a result.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map1 = StableMap::new();
    /// assert_eq!(map1.insert(1, "a"), None);
    /// assert_eq!(map1.insert(2, "b"), None);
    /// assert_eq!(map1.insert(3, "c"), None);
    /// assert_eq!(map1.len(), 3);
    ///
    /// let mut map2 = StableMap::new();
    ///
    /// for (key, value) in map1.into_iter() {
    ///     unsafe {
    ///         map2.insert_unique_unchecked(key, value);
    ///     }
    /// }
    ///
    /// let (key, value) = unsafe { map2.insert_unique_unchecked(4, "d") };
    /// assert_eq!(key, &4);
    /// assert_eq!(value, &mut "d");
    /// *value = "e";
    ///
    /// assert_eq!(map2[&1], "a");
    /// assert_eq!(map2[&2], "b");
    /// assert_eq!(map2[&3], "c");
    /// assert_eq!(map2[&4], "e");
    /// assert_eq!(map2.len(), 4);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub unsafe fn insert_unique_unchecked(&mut self, key: K, value: V) -> (&K, &mut V)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        let pos = self.storage.insert(value);
        let (key, pos) = unsafe {
            // SAFETY:
            // - The requirement is forwarded to the caller.
            self.key_to_pos.insert_unique_unchecked(key, pos)
        };
        let value = unsafe {
            // SAFETY:
            // - We just retrieved this position.
            self.storage.get_unchecked_mut(pos)
        };
        (key, value)
    }

    /// Creates a consuming iterator visiting all the keys in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// let mut vec: Vec<&str> = map.into_keys().collect();
    ///
    /// // The `IntoKeys` iterator produces keys in arbitrary order, so the
    /// // keys must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, ["a", "b", "c"]);
    /// ```
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K> {
        IntoKeys {
            iter: self.key_to_pos.into_keys(),
        }
    }

    /// Creates a consuming iterator visiting all the values in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// let mut vec: Vec<i32> = map.into_values().collect();
    ///
    /// // The `IntoValues` iterator produces values in arbitrary order, so
    /// // the values must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues {
            iter: self.key_to_pos.into_values(),
            storage: self.storage,
        }
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn is_empty(&self) -> bool {
        self.key_to_pos.is_empty()
    }

    /// Returns `true` if the map contains elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert!(!a.is_not_empty());
    /// a.insert(1, "a");
    /// assert!(a.is_not_empty());
    /// ```
    pub fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    /// assert_eq!(map.len(), 3);
    /// let mut vec: Vec<(&str, i32)> = Vec::new();
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {} val: {}", key, val);
    ///     vec.push((*key, *val));
    /// }
    ///
    /// // The `Iter` iterator produces items in arbitrary order, so the
    /// // items must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, [("a", 1), ("b", 2), ("c", 3)]);
    ///
    /// assert_eq!(map.len(), 3);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.key_to_pos.iter(),
            entries: &self.storage,
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// assert_eq!(map.len(), 3);
    /// let mut vec: Vec<(&str, i32)> = Vec::new();
    ///
    /// for (key, val) in &map {
    ///     println!("key: {} val: {}", key, val);
    ///     vec.push((*key, *val));
    /// }
    ///
    /// // The `Iter` iterator produces items in arbitrary order, so the
    /// // items must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, [("a", 2), ("b", 4), ("c", 6)]);
    ///
    /// assert_eq!(map.len(), 3);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.key_to_pos.iter_mut(),
            entries: self.storage.raw_access(),
        }
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    /// assert_eq!(map.len(), 3);
    /// let mut vec: Vec<&str> = Vec::new();
    ///
    /// for key in map.keys() {
    ///     println!("{}", key);
    ///     vec.push(*key);
    /// }
    ///
    /// // The `Keys` iterator produces keys in arbitrary order, so the
    /// // keys must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, ["a", "b", "c"]);
    ///
    /// assert_eq!(map.len(), 3);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn keys(&self) -> Keys<'_, K> {
        Keys {
            iter: self.key_to_pos.keys(),
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> usize {
        self.key_to_pos.len()
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map. Keeps the allocated memory for reuse.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// // The map is empty
    /// assert!(map.is_empty() && map.capacity() == 0);
    ///
    /// map.insert(1, "a");
    ///
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    ///
    /// // Now map holds none elements
    /// assert!(map.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let pos = self.key_to_pos.remove(key)?;
        let value = unsafe {
            // SAFETY:
            // - By the invariants, pos is valid
            self.storage.take_unchecked(pos)
        };
        Some(value)
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map. Keeps the allocated memory for reuse.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// // The map is empty
    /// assert!(map.is_empty() && map.capacity() == 0);
    ///
    /// map.insert(1, "a");
    ///
    /// assert_eq!(map.remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.remove(&1), None);
    ///
    /// // Now map hold none elements
    /// assert!(map.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
        S: BuildHasher,
    {
        let (k, pos) = self.key_to_pos.remove_entry(key)?;
        let value = unsafe {
            // SAFETY:
            // - By the invariants, pos is valid
            self.storage.take_unchecked(pos)
        };
        Some((k, value))
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `StableMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds [`isize::MAX`] bytes and [`abort`] the program
    /// in case of allocation error.
    ///
    /// [`isize::MAX`]: https://doc.rust-lang.org/std/primitive.isize.html
    /// [`abort`]: https://doc.rust-lang.org/alloc/alloc/fn.handle_alloc_error.html
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    /// let mut map: StableMap<&str, i32> = StableMap::new();
    /// // Map is empty and doesn't allocate memory
    /// assert_eq!(map.capacity(), 0);
    ///
    /// map.reserve(10);
    ///
    /// // And now map can hold at least 10 elements
    /// assert!(map.capacity() >= 10);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn reserve(&mut self, additional: usize)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.key_to_pos.reserve(additional);
        self.storage.reserve(additional);
    }

    /// Retains only the elements specified by the predicate. Keeps the
    /// allocated memory for reuse.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)` returns `false`.
    /// The elements are visited in unsorted (and unspecified) order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<i32, i32> = (0..8).map(|x|(x, x*10)).collect();
    /// assert_eq!(map.len(), 8);
    ///
    /// map.retain(|&k, _| k % 2 == 0);
    ///
    /// // We can see, that the number of elements inside map is changed.
    /// assert_eq!(map.len(), 4);
    ///
    /// let mut vec: Vec<(i32, i32)> = map.iter().map(|(&k, &v)| (k, v)).collect();
    /// vec.sort_unstable();
    /// assert_eq!(vec, [(0, 0), (2, 20), (4, 40), (6, 60)]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        let storage = &raw mut self.storage;
        let iter = self.key_to_pos.extract_if(move |k, pos| {
            let storage = unsafe {
                // SAFETY: See the documentation in extract_if
                &mut *storage
            };
            let value = unsafe {
                // SAFETY: By the invariants, pos is valid
                storage.get_unchecked_mut(pos)
            };
            let retain = f(k, value);
            !retain
        });
        for (_, pos) in iter {
            let storage = unsafe {
                // SAFETY: See the documentation in extract_if
                &mut *storage
            };
            unsafe {
                // SAFETY: By the invariants, pos is valid
                storage.take_unchecked(pos);
            }
        }
    }

    /// Shrinks the capacity of the map as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<i32, i32> = StableMap::with_capacity(100);
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// assert!(map.capacity() >= 100);
    /// map.shrink_to_fit();
    /// assert!(map.capacity() >= 2);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn shrink_to_fit(&mut self)
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        self.key_to_pos.shrink_to_fit();
        self.storage.shrink_to_fit();
    }

    /// Tries to insert a key-value pair into the map, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Errors
    ///
    /// If the map already had this key present, nothing is updated, and
    /// an error containing the occupied entry and the value is returned.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use stable_map::{OccupiedError, StableMap};
    ///
    /// let mut map = StableMap::new();
    /// assert_eq!(map.try_insert(37, "a").unwrap(), &"a");
    ///
    /// match map.try_insert(37, "b") {
    ///     Err(OccupiedError { entry, value }) => {
    ///         assert_eq!(entry.key(), &37);
    ///         assert_eq!(entry.get(), &"a");
    ///         assert_eq!(value, "b");
    ///     }
    ///     _ => panic!()
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn try_insert(&mut self, key: K, value: V) -> Result<&mut V, OccupiedError<'_, K, V, S>>
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        match self.entry(key) {
            Entry::Occupied(o) => Err(OccupiedError { entry: o, value }),
            Entry::Vacant(v) => Ok(v.insert(value)),
        }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    /// assert_eq!(map.len(), 3);
    /// let mut vec: Vec<i32> = Vec::new();
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    ///     vec.push(*val);
    /// }
    ///
    /// // The `Values` iterator produces values in arbitrary order, so the
    /// // values must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, [1, 2, 3]);
    ///
    /// assert_eq!(map.len(), 3);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.key_to_pos.values(),
            storage: &self.storage,
        }
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    ///
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values_mut() {
    ///     *val = *val + 10;
    /// }
    ///
    /// assert_eq!(map.len(), 3);
    /// let mut vec: Vec<i32> = Vec::new();
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    ///     vec.push(*val);
    /// }
    ///
    /// // The `Values` iterator produces values in arbitrary order, so the
    /// // values must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, [11, 12, 13]);
    ///
    /// assert_eq!(map.len(), 3);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.key_to_pos.values_mut(),
            storage: self.storage.raw_access(),
        }
    }

    /// Creates an empty `StableMap` with the specified capacity, using `hash_builder`
    /// to hash the keys.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::DefaultHashBuilder;
    /// use stable_map::StableMap;
    ///
    /// let s = DefaultHashBuilder::default();
    /// let mut map = StableMap::with_capacity_and_hasher(10, s);
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 10);
    ///
    /// map.insert(1, 2);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            key_to_pos: HashMap::with_capacity_and_hasher(capacity, hash_builder),
            storage: LinearStorage::with_capacity(capacity),
        }
    }

    /// Creates an empty `StableMap` which will use the given hash builder to hash
    /// keys.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not
    /// allocate until it is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::DefaultHashBuilder;
    /// use stable_map::StableMap;
    ///
    /// let s = DefaultHashBuilder::default();
    /// let mut map = StableMap::with_hasher(s);
    /// assert_eq!(map.len(), 0);
    /// assert_eq!(map.capacity(), 0);
    ///
    /// map.insert(1, 2);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            key_to_pos: HashMap::with_hasher(hash_builder),
            storage: LinearStorage::with_capacity(0),
        }
    }

    /// Returns one more than the highest possible index of this map.
    ///
    /// Using [get_by_index](Self::get_by_index) with higher indices will always return
    /// `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert_eq!(a.index_len(), 0);
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.remove(&2);
    /// assert_eq!(a.len(), 1);
    /// assert_eq!(a.index_len(), 2);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn index_len(&self) -> usize {
        self.storage.len()
    }

    /// Returns the index that the key maps to.
    ///
    /// This function returns `Some` if and only if the key is contained in the map.
    ///
    /// As long as the key is not removed from the map, and unless
    /// [compact](Self::compact) or [force_compact](Self::force_compact) is called, this
    /// function will always return the same value.
    ///
    /// The returned value can be used to retrieve the value by using
    /// [get_by_index](Self::get_by_index) or [get_by_index_mut](Self::get_by_index_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert_eq!(a.index_len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.get_by_index(a.get_index(&1).unwrap()).unwrap(), &"a");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn get_index<Q>(&self, q: &Q) -> Option<usize>
    where
        S: BuildHasher,
        K: Eq + Hash,
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.key_to_pos.get(q).map(|v| unsafe {
            // SAFETY:
            // - By the invariants, v is valid
            v.get_unchecked()
        })
    }

    /// Returns a reference to the value corresponding to the index.
    ///
    /// This function returns `Some` if and only if there is a key, `key`, for which
    /// [get_index](Self::get_index) returns this index. In this case, it returns the same
    /// value that would be returned by calling [get](Self::get).
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert_eq!(a.index_len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.get_by_index(a.get_index(&1).unwrap()).unwrap(), &"a");
    /// ```
    #[inline]
    pub fn get_by_index(&self, index: usize) -> Option<&V> {
        self.storage.get(index)
    }

    /// Returns a mutable reference to the value corresponding to the index.
    ///
    /// This function returns `Some` if and only if there is a key, `key`, for which
    /// [get_index](Self::get_index) returns this index. In this case, it returns the same
    /// value that would be returned by calling [get_mut](Self::get_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert_eq!(a.index_len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.get_by_index_mut(a.get_index(&1).unwrap()).unwrap(), &"a");
    /// ```
    #[inline]
    pub fn get_by_index_mut(&mut self, index: usize) -> Option<&mut V> {
        self.storage.get_mut(index)
    }

    /// Returns a reference to the value corresponding to the index, without
    /// validating that the index is valid.
    ///
    /// This function returns the same value that would be returned by
    /// [get_by_index](Self::get_by_index).
    ///
    /// # Safety
    ///
    /// There must be some `key` for which `self.get_index(k)` would return this index.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert_eq!(a.index_len(), 0);
    /// a.insert(1, "a");
    /// unsafe {
    ///     assert_eq!(a.get_by_index_unchecked(a.get_index(&1).unwrap()), &"a");
    /// }
    /// ```
    #[inline]
    pub unsafe fn get_by_index_unchecked(&self, index: usize) -> &V {
        unsafe {
            // SAFETY:
            // - By the requirements of this function, there is an element of key_to_pos
            //   with this index.
            // - By the invariants, that element could be used to call get_unchecked.
            self.storage.get_unchecked_raw(index)
        }
    }

    /// Returns a mutable reference to the value corresponding to the index, without
    /// validating that the index is valid.
    ///
    /// This function returns the same value that would be returned by
    /// [get_by_index_mut](Self::get_by_index_mut).
    ///
    /// # Safety
    ///
    /// There must be some `key` for which `self.get_index(k)` would return this index.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut a = StableMap::new();
    /// assert_eq!(a.index_len(), 0);
    /// a.insert(1, "a");
    /// unsafe {
    ///     assert_eq!(a.get_by_index_unchecked_mut(a.get_index(&1).unwrap()), &"a");
    /// }
    /// ```
    #[inline]
    pub unsafe fn get_by_index_unchecked_mut(&mut self, index: usize) -> &mut V {
        unsafe {
            // SAFETY:
            // - By the requirements of this function, there is an element of key_to_pos
            //   with this index.
            // - By the invariants, that element could be used to call get_unchecked_mut.
            self.storage.get_unchecked_raw_mut(index)
        }
    }

    /// Maybe compacts the map, removing indices for which `get_by_index` would return
    /// `None`.
    ///
    /// This function does nothing if there are no more than 8 indices for which
    /// [get_by_index](Self::get_by_index) returns `None` or if at least half of the
    /// indices are in use.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// for i in 0..32 {
    ///     map.insert(i, i);
    /// }
    /// for i in 0..16 {
    ///     map.remove(&i);
    /// }
    /// assert_eq!(map.index_len(), 32);
    /// map.compact();
    /// assert_eq!(map.index_len(), 32);
    /// map.remove(&16);
    /// map.compact();
    /// assert_eq!(map.index_len(), 15);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn compact(&mut self) {
        self.storage.compact();
    }

    /// Compacts the map, removing indices for which `get_by_index` would return `None`.
    ///
    /// After this function returns, [index_len](Self::index_len) will be the same as
    /// [len](Self::len).
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map = StableMap::new();
    /// map.insert(1, 1);
    /// map.remove(&1);
    /// assert_eq!(map.index_len(), 1);
    /// map.force_compact();
    /// assert_eq!(map.index_len(), 0);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn force_compact(&mut self) {
        self.storage.force_compact();
    }
}

impl<K, V, S> IntoIterator for StableMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.key_to_pos.into_iter(),
            storage: self.storage,
        }
    }
}

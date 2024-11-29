#[cfg(test)]
mod tests;

use {
    crate::{
        linear_storage::LinearStorage,
        pos_vec::pos::{InUse, Pos},
    },
    core::{
        borrow::Borrow,
        fmt::{Debug, Formatter},
        hash::{BuildHasher, Hash},
        mem::{self},
    },
    hashbrown::hash_map,
};

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`StableMap`].
///
/// [`StableMap`]: crate::StableMap
/// [`entry`]: crate::StableMap::entry
///
/// # Examples
///
/// ```
/// use stable_map::{Entry, StableMap, OccupiedEntry};
///
/// let mut map = StableMap::new();
/// map.extend([("a", 10), ("b", 20), ("c", 30)]);
/// assert_eq!(map.len(), 3);
///
/// // Existing key (insert)
/// let entry: Entry<_, _, _> = map.entry("a");
/// let _raw_o: OccupiedEntry<_, _, _> = entry.insert(1);
/// assert_eq!(map.len(), 3);
/// // Nonexistent key (insert)
/// map.entry("d").insert(4);
///
/// // Existing key (or_insert)
/// let v = map.entry("b").or_insert(2);
/// assert_eq!(std::mem::replace(v, 2), 20);
/// // Nonexistent key (or_insert)
/// map.entry("e").or_insert(5);
///
/// // Existing key (or_insert_with)
/// let v = map.entry("c").or_insert_with(|| 3);
/// assert_eq!(std::mem::replace(v, 3), 30);
/// // Nonexistent key (or_insert_with)
/// map.entry("f").or_insert_with(|| 6);
///
/// println!("Our StableMap: {:?}", map);
///
/// let mut vec: Vec<_> = map.iter().map(|(&k, &v)| (k, v)).collect();
/// // The `Iter` iterator produces items in arbitrary order, so the
/// // items must be sorted to test them against a sorted array.
/// vec.sort_unstable();
/// assert_eq!(vec, [("a", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5), ("f", 6)]);
/// ```
pub enum Entry<'a, K, V, S> {
    /// An occupied entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    /// let mut map: StableMap<_, _> = [("a", 100), ("b", 200)].into();
    ///
    /// match map.entry("a") {
    ///     Entry::Vacant(_) => unreachable!(),
    ///     Entry::Occupied(_) => { }
    /// }
    /// ```
    Occupied(OccupiedEntry<'a, K, V, S>),
    /// A vacant entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    /// let mut map: StableMap<&str, i32> = StableMap::new();
    ///
    /// match map.entry("a") {
    ///     Entry::Occupied(_) => unreachable!(),
    ///     Entry::Vacant(_) => { }
    /// }
    /// ```
    Vacant(VacantEntry<'a, K, V, S>),
}

/// A view into a single entry in a map, which may either be vacant or occupied,
/// with any borrowed form of the map's key type.
///
///
/// This `enum` is constructed from the [`entry_ref`] method on [`StableMap`].
///
/// [`Hash`] and [`Eq`] on the borrowed form of the map's key type *must* match those
/// for the key type. It also require that key may be constructed from the borrowed
/// form through the [`From`] trait.
///
/// [`StableMap`]: crate::StableMap
/// [`entry_ref`]: crate::StableMap::entry_ref
/// [`Eq`]: https://doc.rust-lang.org/std/cmp/trait.Eq.html
/// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
/// [`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
///
/// # Examples
///
/// ```
/// use stable_map::{EntryRef, StableMap, OccupiedEntry};
///
/// let mut map = StableMap::new();
/// map.extend([("a".to_owned(), 10), ("b".into(), 20), ("c".into(), 30)]);
/// assert_eq!(map.len(), 3);
///
/// // Existing key (insert)
/// let key = String::from("a");
/// let entry: EntryRef<_, _, _, _> = map.entry_ref(&key);
/// let _raw_o: OccupiedEntry<_, _, _> = entry.insert(1);
/// assert_eq!(map.len(), 3);
/// // Nonexistent key (insert)
/// map.entry_ref("d").insert(4);
///
/// // Existing key (or_insert)
/// let v = map.entry_ref("b").or_insert(2);
/// assert_eq!(std::mem::replace(v, 2), 20);
/// // Nonexistent key (or_insert)
/// map.entry_ref("e").or_insert(5);
///
/// // Existing key (or_insert_with)
/// let v = map.entry_ref("c").or_insert_with(|| 3);
/// assert_eq!(std::mem::replace(v, 3), 30);
/// // Nonexistent key (or_insert_with)
/// map.entry_ref("f").or_insert_with(|| 6);
///
/// println!("Our StableMap: {:?}", map);
///
/// for (key, value) in ["a", "b", "c", "d", "e", "f"].into_iter().zip(1..=6) {
///     assert_eq!(map[key], value)
/// }
/// assert_eq!(map.len(), 6);
/// ```
pub enum EntryRef<'a, 'b, K, Q: ?Sized, V, S> {
    /// An occupied entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{EntryRef, StableMap};
    /// let mut map: StableMap<_, _> = [("a".to_owned(), 100), ("b".into(), 200)].into();
    ///
    /// match map.entry_ref("a") {
    ///     EntryRef::Vacant(_) => unreachable!(),
    ///     EntryRef::Occupied(_) => { }
    /// }
    /// ```
    Occupied(OccupiedEntry<'a, K, V, S>),
    /// A vacant entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{EntryRef, StableMap};
    /// let mut map: StableMap<String, i32> = StableMap::new();
    ///
    /// match map.entry_ref("a") {
    ///     EntryRef::Occupied(_) => unreachable!(),
    ///     EntryRef::Vacant(_) => { }
    /// }
    /// ```
    Vacant(VacantEntryRef<'a, 'b, K, Q, V, S>),
}

/// A view into an occupied entry in a [`StableMap`](crate::StableMap).
/// It is part of the [`Entry`] and [`EntryRef`] enums.
///
/// # Examples
///
/// ```
/// use stable_map::{Entry, StableMap, OccupiedEntry};
///
/// let mut map = StableMap::new();
/// map.extend([("a", 10), ("b", 20), ("c", 30)]);
///
/// let _entry_o: OccupiedEntry<_, _, _> = map.entry("a").insert(100);
/// assert_eq!(map.len(), 3);
///
/// // Existing key (insert and update)
/// match map.entry("a") {
///     Entry::Vacant(_) => unreachable!(),
///     Entry::Occupied(mut view) => {
///         assert_eq!(view.get(), &100);
///         let v = view.get_mut();
///         *v *= 10;
///         assert_eq!(view.insert(1111), 1000);
///     }
/// }
///
/// assert_eq!(map[&"a"], 1111);
/// assert_eq!(map.len(), 3);
///
/// // Existing key (take)
/// match map.entry("c") {
///     Entry::Vacant(_) => unreachable!(),
///     Entry::Occupied(view) => {
///         assert_eq!(view.remove_entry(), ("c", 30));
///     }
/// }
/// assert_eq!(map.get(&"c"), None);
/// assert_eq!(map.len(), 2);
/// ```
pub struct OccupiedEntry<'a, K, V, S> {
    pub(crate) entry: hash_map::OccupiedEntry<'a, K, Pos<InUse>, S>,
    pub(crate) entries: &'a mut LinearStorage<V>,
}

/// A view into a vacant entry in a `StableMap`.
/// It is part of the [`Entry`] enum.
///
/// # Examples
///
/// ```
/// use stable_map::{Entry, StableMap, VacantEntry};
///
/// let mut map = StableMap::<&str, i32>::new();
///
/// let entry_v: VacantEntry<_, _, _> = match map.entry("a") {
///     Entry::Vacant(view) => view,
///     Entry::Occupied(_) => unreachable!(),
/// };
/// entry_v.insert(10);
/// assert!(map[&"a"] == 10 && map.len() == 1);
///
/// // Nonexistent key (insert and update)
/// match map.entry("b") {
///     Entry::Occupied(_) => unreachable!(),
///     Entry::Vacant(view) => {
///         let value = view.insert(2);
///         assert_eq!(*value, 2);
///         *value = 20;
///     }
/// }
/// assert!(map[&"b"] == 20 && map.len() == 2);
/// ```
pub struct VacantEntry<'a, K, V, S> {
    pub(crate) entry: hash_map::VacantEntry<'a, K, Pos<InUse>, S>,
    pub(crate) entries: &'a mut LinearStorage<V>,
}

/// A view into a vacant entry in a `StableMap`.
/// It is part of the [`EntryRef`] enum.
///
/// # Examples
///
/// ```
/// use stable_map::{EntryRef, StableMap, VacantEntryRef};
///
/// let mut map = StableMap::<String, i32>::new();
///
/// let entry_v: VacantEntryRef<_, _, _, _> = match map.entry_ref("a") {
///     EntryRef::Vacant(view) => view,
///     EntryRef::Occupied(_) => unreachable!(),
/// };
/// entry_v.insert(10);
/// assert!(map["a"] == 10 && map.len() == 1);
///
/// // Nonexistent key (insert and update)
/// match map.entry_ref("b") {
///     EntryRef::Occupied(_) => unreachable!(),
///     EntryRef::Vacant(view) => {
///         let value = view.insert(2);
///         assert_eq!(*value, 2);
///         *value = 20;
///     }
/// }
/// assert!(map["b"] == 20 && map.len() == 2);
/// ```
pub struct VacantEntryRef<'a, 'b, K, Q, V, S>
where
    Q: ?Sized,
{
    pub(crate) entry: hash_map::VacantEntryRef<'a, 'b, K, Q, Pos<InUse>, S>,
    pub(crate) entries: &'a mut LinearStorage<V>,
}

impl<'a, K, V, S> OccupiedEntry<'a, K, V, S> {
    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// match map.entry("poneyland") {
    ///     Entry::Vacant(_) => panic!(),
    ///     Entry::Occupied(entry) => assert_eq!(entry.get(), &12),
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn get(&self) -> &V {
        unsafe {
            // SAFETY: By the invariants, self.entry.get() is valid.
            self.entries.get_unchecked(self.entry.get())
        }
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`](Self::into_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     *o.get_mut() += 10;
    ///     assert_eq!(*o.get(), 22);
    ///
    ///     // We can use the same Entry multiple times.
    ///     *o.get_mut() += 2;
    /// }
    ///
    /// assert_eq!(map["poneyland"], 24);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn get_mut(&mut self) -> &mut V {
        unsafe {
            // SAFETY: By the invariants, self.entry.get() is valid.
            self.entries.get_unchecked_mut(self.entry.get())
        }
    }

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     assert_eq!(o.insert(15), 12);
    /// }
    ///
    /// assert_eq!(map["poneyland"], 15);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`](Self::get_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    ///
    /// let value: &mut u32;
    /// match map.entry("poneyland") {
    ///     Entry::Occupied(entry) => value = entry.into_mut(),
    ///     Entry::Vacant(_) => panic!(),
    /// }
    /// *value += 10;
    ///
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn into_mut(self) -> &'a mut V {
        unsafe {
            // SAFETY: By the invariants, self.entry.get() is valid.
            self.entries.get_unchecked_mut(self.entry.get())
        }
    }

    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// match map.entry("poneyland") {
    ///     Entry::Vacant(_) => panic!(),
    ///     Entry::Occupied(entry) => assert_eq!(entry.key(), &"poneyland"),
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn key(&self) -> &K {
        self.entry.key()
    }

    /// Takes the value out of the entry, and returns it.
    /// Keeps the allocated memory for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// // The map is empty
    /// assert!(map.is_empty() && map.capacity() == 0);
    ///
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.remove(), 12);
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// // Now map hold none elements
    /// assert!(map.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove(self) -> V {
        let pos = self.entry.remove();
        unsafe {
            // SAFETY: By the invariants, self.entry.get() is valid.
            self.entries.take_unchecked(pos)
        }
    }

    /// Take the ownership of the key and value from the map.
    /// Keeps the allocated memory for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// // The map is empty
    /// assert!(map.is_empty() && map.capacity() == 0);
    ///
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     // We delete the entry from the map.
    ///     assert_eq!(o.remove_entry(), ("poneyland", 12));
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// // Now map hold none elements
    /// assert!(map.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove_entry(self) -> (K, V) {
        let (k, pos) = self.entry.remove_entry();
        let value = unsafe {
            // SAFETY: By the invariants, self.entry.get() is valid.
            self.entries.take_unchecked(pos)
        };
        (k, value)
    }

    /// Provides shared access to the key and owned access to the value of
    /// the entry and allows to replace or remove it based on the
    /// value of the returned option.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// map.insert("poneyland", 42);
    ///
    /// let entry = match map.entry("poneyland") {
    ///     Entry::Occupied(e) => {
    ///         e.replace_entry_with(|k, v| {
    ///             assert_eq!(k, &"poneyland");
    ///             assert_eq!(v, 42);
    ///             Some(v + 1)
    ///         })
    ///     }
    ///     Entry::Vacant(_) => panic!(),
    /// };
    ///
    /// match entry {
    ///     Entry::Occupied(e) => {
    ///         assert_eq!(e.key(), &"poneyland");
    ///         assert_eq!(e.get(), &43);
    ///     }
    ///     Entry::Vacant(_) => panic!(),
    /// }
    ///
    /// assert_eq!(map["poneyland"], 43);
    ///
    /// let entry = match map.entry("poneyland") {
    ///     Entry::Occupied(e) => e.replace_entry_with(|_k, _v| None),
    ///     Entry::Vacant(_) => panic!(),
    /// };
    ///
    /// match entry {
    ///     Entry::Vacant(e) => {
    ///         assert_eq!(e.key(), &"poneyland");
    ///     }
    ///     Entry::Occupied(_) => panic!(),
    /// }
    ///
    /// assert!(!map.contains_key("poneyland"));
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn replace_entry_with<F>(self, f: F) -> Entry<'a, K, V, S>
    where
        F: FnOnce(&K, V) -> Option<V>,
    {
        let entry = self.entry.replace_entry_with(|k, pos| {
            let v = unsafe { self.entries.take_unchecked(pos) };
            match f(k, v) {
                None => None,
                Some(v) => Some(self.entries.insert(v)),
            }
        });
        match entry {
            hash_map::Entry::Occupied(o) => Entry::Occupied(OccupiedEntry {
                entry: o,
                entries: self.entries,
            }),
            hash_map::Entry::Vacant(v) => Entry::Vacant(VacantEntry {
                entry: v,
                entries: self.entries,
            }),
        }
    }
}

impl<'a, K, V, S> VacantEntry<'a, K, V, S> {
    /// Sets the value of the entry with the [`VacantEntry`]'s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// if let Entry::Vacant(o) = map.entry("poneyland") {
    ///     o.insert(37);
    /// }
    /// assert_eq!(map["poneyland"], 37);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash,
        S: BuildHasher,
    {
        let pos = self.entries.insert(value);
        let pos = self.entry.insert(pos);
        unsafe { self.entries.get_unchecked_mut(pos) }
    }

    /// Sets the value of the entry with the [`VacantEntry`]'s key,
    /// and returns an [`OccupiedEntry`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// if let Entry::Vacant(v) = map.entry("poneyland") {
    ///     let o = v.insert_entry(37);
    ///     assert_eq!(o.get(), &37);
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, S>
    where
        K: Hash,
        S: BuildHasher,
    {
        let pos = self.entries.insert(value);
        let entry = self.entry.insert_entry(pos);
        OccupiedEntry {
            entry,
            entries: self.entries,
        }
    }

    /// Take ownership of the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// match map.entry("poneyland") {
    ///     Entry::Occupied(_) => panic!(),
    ///     Entry::Vacant(v) => assert_eq!(v.into_key(), "poneyland"),
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn into_key(self) -> K {
        self.entry.into_key()
    }

    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn key(&self) -> &K {
        self.entry.key()
    }
}

impl<'a, K, V, S> Entry<'a, K, V, S> {
    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Entry::Occupied(e) = &mut self {
            f(e.get_mut());
        }
        self
    }

    /// Provides shared access to the key and owned access to the value of
    /// an occupied entry and allows to replace or remove it based on the
    /// value of the returned option.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{Entry, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// let entry = map
    ///     .entry("poneyland")
    ///     .and_replace_entry_with(|_k, _v| panic!());
    ///
    /// match entry {
    ///     Entry::Vacant(e) => {
    ///         assert_eq!(e.key(), &"poneyland");
    ///     }
    ///     Entry::Occupied(_) => panic!(),
    /// }
    ///
    /// map.insert("poneyland", 42);
    ///
    /// let entry = map
    ///     .entry("poneyland")
    ///     .and_replace_entry_with(|k, v| {
    ///         assert_eq!(k, &"poneyland");
    ///         assert_eq!(v, 42);
    ///         Some(v + 1)
    ///     });
    ///
    /// match entry {
    ///     Entry::Occupied(e) => {
    ///         assert_eq!(e.key(), &"poneyland");
    ///         assert_eq!(e.get(), &43);
    ///     }
    ///     Entry::Vacant(_) => panic!(),
    /// }
    ///
    /// assert_eq!(map["poneyland"], 43);
    ///
    /// let entry = map
    ///     .entry("poneyland")
    ///     .and_replace_entry_with(|_k, _v| None);
    ///
    /// match entry {
    ///     Entry::Vacant(e) => assert_eq!(e.key(), &"poneyland"),
    ///     Entry::Occupied(_) => panic!(),
    /// }
    ///
    /// assert!(!map.contains_key("poneyland"));
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn and_replace_entry_with<F>(self, f: F) -> Self
    where
        F: FnOnce(&K, V) -> Option<V>,
    {
        match self {
            Entry::Occupied(o) => o.replace_entry_with(f),
            Entry::Vacant(v) => Entry::Vacant(v),
        }
    }

    /// Sets the value of the entry, and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// let entry = map.entry("horseyland").insert(37);
    ///
    /// assert_eq!(entry.key(), &"horseyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(self, value: V) -> OccupiedEntry<'a, K, V, S>
    where
        K: Hash,
        S: BuildHasher,
    {
        match self {
            Entry::Occupied(mut o) => {
                o.insert(value);
                o
            }
            Entry::Vacant(v) => v.insert_entry(value),
        }
    }

    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    /// map.entry("poneyland").or_insert(3);
    /// // existing key
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// // nonexistent key
    /// assert_eq!(map.entry("horseland").key(), &"horseland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(e) => e.key(),
            Entry::Vacant(e) => e.key(),
        }
    }

    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, Option<u32>> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry("poneyland").or_default();
    /// assert_eq!(map["poneyland"], None);
    ///
    /// map.insert("horseland", Some(3));
    ///
    /// // existing key
    /// assert_eq!(map.entry("horseland").or_default(), &mut Some(3));
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_default(self) -> &'a mut V
    where
        K: Hash,
        S: BuildHasher,
        V: Default,
    {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(V::default()),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry("poneyland").or_insert(3);
    /// assert_eq!(map["poneyland"], 3);
    ///
    /// // existing key
    /// *map.entry("poneyland").or_insert(10) *= 2;
    /// assert_eq!(map["poneyland"], 6);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert(self, value: V) -> &'a mut V
    where
        K: Hash,
        S: BuildHasher,
    {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(value),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry("poneyland").or_insert_with(|| 3);
    /// assert_eq!(map["poneyland"], 3);
    ///
    /// // existing key
    /// *map.entry("poneyland").or_insert_with(|| 10) *= 2;
    /// assert_eq!(map["poneyland"], 6);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        K: Hash,
        S: BuildHasher,
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(f()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<&str, usize> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry("poneyland").or_insert_with_key(|key| key.chars().count());
    /// assert_eq!(map["poneyland"], 9);
    ///
    /// // existing key
    /// *map.entry("poneyland").or_insert_with_key(|key| key.chars().count() * 10) *= 2;
    /// assert_eq!(map["poneyland"], 18);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert_with_key<F>(self, f: F) -> &'a mut V
    where
        K: Hash,
        S: BuildHasher,
        F: FnOnce(&K) -> V,
    {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let value = f(v.key());
                v.insert(value)
            }
        }
    }
}

impl<'a, 'b, K, Q, V, S> VacantEntryRef<'a, 'b, K, Q, V, S>
where
    Q: ?Sized,
{
    /// Sets the value of the entry, and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, u32> = StableMap::new();
    /// let entry = map.entry_ref("horseyland").insert(37);
    ///
    /// assert_eq!(entry.key(), "horseyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash + From<&'b Q>,
        S: BuildHasher,
    {
        let pos = self.entries.insert(value);
        let pos = self.entry.insert(pos);
        unsafe { self.entries.get_unchecked_mut(pos) }
    }

    /// Sets the value of the entry with the [`VacantEntryRef`]'s key,
    /// and returns an [`OccupiedEntry`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::{EntryRef, StableMap};
    ///
    /// let mut map: StableMap<&str, u32> = StableMap::new();
    ///
    /// if let EntryRef::Vacant(v) = map.entry_ref("poneyland") {
    ///     let o = v.insert_entry(37);
    ///     assert_eq!(o.get(), &37);
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, S>
    where
        K: Hash + From<&'b Q>,
        S: BuildHasher,
    {
        let pos = self.entries.insert(value);
        let entry = self.entry.insert_entry(pos);
        OccupiedEntry {
            entry,
            entries: self.entries,
        }
    }

    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntryRef`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, u32> = StableMap::new();
    /// let key: &str = "poneyland";
    /// assert_eq!(map.entry_ref(key).key(), "poneyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn key(&self) -> &'b Q {
        self.entry.key()
    }
}

impl<'a, 'b, K, Q, V, S> EntryRef<'a, 'b, K, Q, V, S>
where
    Q: ?Sized,
{
    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, u32> = StableMap::new();
    ///
    /// map.entry_ref("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry_ref("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let EntryRef::Occupied(e) = &mut self {
            f(e.get_mut());
        }
        self
    }

    /// Sets the value of the entry, and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, u32> = StableMap::new();
    /// let entry = map.entry_ref("horseyland").insert(37);
    ///
    /// assert_eq!(entry.key(), "horseyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(self, value: V) -> OccupiedEntry<'a, K, V, S>
    where
        K: Hash + From<&'b Q>,
        S: BuildHasher,
    {
        match self {
            EntryRef::Occupied(mut o) => {
                o.insert(value);
                o
            }
            EntryRef::Vacant(v) => v.insert_entry(value),
        }
    }

    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, u32> = StableMap::new();
    /// map.entry_ref("poneyland").or_insert(3);
    /// // existing key
    /// assert_eq!(map.entry_ref("poneyland").key(), "poneyland");
    /// // nonexistent key
    /// assert_eq!(map.entry_ref("horseland").key(), "horseland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn key(&self) -> &Q
    where
        K: Borrow<Q>,
    {
        match self {
            EntryRef::Occupied(e) => e.key().borrow(),
            EntryRef::Vacant(e) => e.key(),
        }
    }

    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, Option<u32>> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry_ref("poneyland").or_default();
    /// assert_eq!(map["poneyland"], None);
    ///
    /// map.insert("horseland".to_string(), Some(3));
    ///
    /// // existing key
    /// assert_eq!(map.entry_ref("horseland").or_default(), &mut Some(3));
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_default(self) -> &'a mut V
    where
        K: Hash + From<&'b Q>,
        S: BuildHasher,
        V: Default,
    {
        match self {
            EntryRef::Occupied(o) => o.into_mut(),
            EntryRef::Vacant(v) => v.insert(V::default()),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, u32> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry_ref("poneyland").or_insert(3);
    /// assert_eq!(map["poneyland"], 3);
    ///
    /// // existing key
    /// *map.entry_ref("poneyland").or_insert(10) *= 2;
    /// assert_eq!(map["poneyland"], 6);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert(self, value: V) -> &'a mut V
    where
        K: Hash + From<&'b Q>,
        S: BuildHasher,
    {
        match self {
            EntryRef::Occupied(o) => o.into_mut(),
            EntryRef::Vacant(v) => v.insert(value),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, u32> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry_ref("poneyland").or_insert_with(|| 3);
    /// assert_eq!(map["poneyland"], 3);
    ///
    /// // existing key
    /// *map.entry_ref("poneyland").or_insert_with(|| 10) *= 2;
    /// assert_eq!(map["poneyland"], 6);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        K: Hash + From<&'b Q>,
        S: BuildHasher,
        F: FnOnce() -> V,
    {
        match self {
            EntryRef::Occupied(o) => o.into_mut(),
            EntryRef::Vacant(v) => v.insert(f()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function an access to the borrower form of the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use stable_map::StableMap;
    ///
    /// let mut map: StableMap<String, usize> = StableMap::new();
    ///
    /// // nonexistent key
    /// map.entry_ref("poneyland").or_insert_with_key(|key| key.chars().count());
    /// assert_eq!(map["poneyland"], 9);
    ///
    /// // existing key
    /// *map.entry_ref("poneyland").or_insert_with_key(|key| key.chars().count() * 10) *= 2;
    /// assert_eq!(map["poneyland"], 18);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert_with_key<F>(self, f: F) -> &'a mut V
    where
        K: Hash + From<&'b Q>,
        S: BuildHasher,
        F: FnOnce(&'b Q) -> V,
    {
        match self {
            EntryRef::Occupied(o) => o.into_mut(),
            EntryRef::Vacant(v) => {
                let value = f(v.key());
                v.insert(value)
            }
        }
    }
}

impl<K, V, S> Debug for OccupiedEntry<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}

impl<K, V, S> Debug for VacantEntry<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VacantEntry").field(self.key()).finish()
    }
}

impl<K, Q, V, S> Debug for VacantEntryRef<'_, '_, K, Q, V, S>
where
    Q: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VacantEntryRef").field(self.key()).finish()
    }
}

impl<K, V, S> Debug for Entry<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            Entry::Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Entry::Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}

impl<K, Q, V, S> Debug for EntryRef<'_, '_, K, Q, V, S>
where
    Q: Debug,
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            EntryRef::Vacant(ref v) => f.debug_tuple("EntryRef").field(v).finish(),
            EntryRef::Occupied(ref o) => f.debug_tuple("EntryRef").field(o).finish(),
        }
    }
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V, S> Send for OccupiedEntry<'_, K, V, S>
where
    K: Send,
    V: Send,
    S: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V, S> Sync for OccupiedEntry<'_, K, V, S>
where
    K: Sync,
    V: Sync,
    S: Sync,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V, S> Send for VacantEntry<'_, K, V, S>
where
    K: Send,
    V: Send,
    S: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, V, S> Sync for VacantEntry<'_, K, V, S>
where
    K: Sync,
    V: Sync,
    S: Sync,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, Q, V, S> Send for VacantEntryRef<'_, '_, K, Q, V, S>
where
    Q: Send,
    K: Send,
    V: Send,
    S: Send,
{
}

// SAFETY:
// - This impl is required because Pos<InUse>, Pos<Stored> allow for conflicting access
//   but this API prevents this.
unsafe impl<K, Q, V, S> Sync for VacantEntryRef<'_, '_, K, Q, V, S>
where
    Q: Sync,
    K: Sync,
    V: Sync,
    S: Sync,
{
}

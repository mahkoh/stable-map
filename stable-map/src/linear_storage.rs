#[cfg(test)]
pub mod tests;

use {
    crate::pos_vec::{
        pos::{Free, InUse, Pos},
        PosVec, PosVecRawAccess,
    },
    min_max_heap::MinMaxHeap,
};

/// A wrapper around a `PosVec` that keeps track of valid `Pos<Free>`.
///
/// Objects of this type return `Pos<InUse>` to the caller. These `Pos`
/// can be passed back into **this** object to perform unchecked access to these
/// positions.
///
/// Some operations cause some of these returned `Pos` to become invalid. When a `Pos`
/// becomes invalid, it must no longer be passed into this object.
//
// This type upholds the following invariants:
//
// - All valid Pos<InUse> are also valid for the underlying PosVec.
// - The free_list contains only valid Pos<Free> returned by the PosVec.
//
// SAFETY: Each mutating function must document how it upholds these invariants.
#[derive(Debug)]
pub struct LinearStorage<V> {
    values: PosVec<V>,
    free_list: MinMaxHeap<Pos<Free>>,
}

impl<V> LinearStorage<V> {
    /// Creates a new vector with the requested capacity.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: PosVec::with_capacity(capacity),
            free_list: Default::default(),
        }
    }

    /// Returns the length of the vector.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns the capacity of the vector.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Stores a value.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(&mut self, value: V) -> Pos<InUse> {
        let pos = match self.free_list.pop_min() {
            Some(pos) => pos,
            _ => self.values.create_pos(),
        };
        unsafe {
            // SAFETY:
            // - If the pos was popped from the free list, then, by the invariants, it
            //   is still valid for self.values.
            // - Otherwise, create_pos, returns a new, valid Pos<Free>.
            self.values.store(pos, value)
        }
        // SAFETY(invariants):
        // - The returned Pos<InUse> was just returned PosVec::store and is therefore still valid.
        // - All Pos<Free> used by this function have been consumed by the PosVec.
    }

    /// Clears the vector.
    ///
    /// This function invalidates all `Pos<InUse>` previously returned by this object.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn clear(&mut self) {
        self.values.clear();
        self.free_list.clear();
        // SAFETY(invariants):
        // - The invalidation of Pos<InUse> is forwarded to the caller.
        // - We've cleared self.free_list.
    }

    /// Retrieves a reference to a value stored at a specific index in the vector.
    ///
    /// Note that, unlike the functions taking `Pos<InUse>`, which value is returned is
    /// affected by calls to `compact`.
    #[inline]
    pub fn get(&self, pos: usize) -> Option<&V> {
        self.values.get(pos)
    }

    /// Retrieves a mutable reference to a value stored at a specific index in the vector.
    ///
    /// Note that, unlike the functions taking `Pos<InUse>`, which value is returned is
    /// affected by calls to `compact`.
    #[inline]
    pub fn get_mut(&mut self, pos: usize) -> Option<&mut V> {
        self.values.get_mut(pos)
    }

    /// Reserves space for `additional` additional elements.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn reserve(&mut self, additional: usize) {
        self.values
            .reserve(additional.saturating_sub(self.free_list.len()));
    }

    /// Reduces the capacity of the vector to its length.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn shrink_to_fit(&mut self) {
        self.values.shrink_to_fit();
    }

    /// Compacts the storage.
    ///
    /// This has no effect if the occupancy is greater than 50% or there are no more than 8 unused
    /// slots.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn compact(&mut self) {
        if self.free_list.len() <= (self.values.len() / 2).max(8) {
            return;
        }
        self.force_compact();
        // SAFETY(invariants):
        // - force_compact ensures that all invariants are upheld.
    }

    /// Compacts the storage unconditionally.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn force_compact(&mut self) {
        unsafe {
            // SAFETY:
            // - By the invariants, free_list contains only valid Pos<Free> returned by self.values.
            self.values.compact(|| self.free_list.pop_min());
        }
        self.free_list.clear();
        // SAFETY(invariants):
        // - This function has no effect on returned Pos<InUse>
        // - We've cleared self.free_list.
    }

    /// Retrieves a reference to the value referenced by a usize.
    ///
    /// # Safety
    ///
    /// There must be a `Pos<InUse>` with the same index that could be used to call
    /// `get_unchecked`.
    #[inline]
    pub unsafe fn get_unchecked_raw(&self, idx: usize) -> &V {
        unsafe {
            // SAFETY:
            // - The requirements are forwarded to the caller.
            // - By the invariants, any Pos<InUse> valid for this object is also valid for
            //   self.values.
            self.values.get_unchecked_raw(idx)
        }
        // SAFETY(invariants):
        // - This function has no effect on the invariants.
    }

    /// Retrieves a mutable reference to the value referenced by a usize.
    ///
    /// # Safety
    ///
    /// There must be a `Pos<InUse>` with the same index that could be used to call
    /// `get_unchecked_mut`.
    #[inline]
    pub unsafe fn get_unchecked_raw_mut(&mut self, idx: usize) -> &mut V {
        unsafe {
            // SAFETY:
            // - The requirements are forwarded to the caller.
            // - By the invariants, any Pos<InUse> valid for this object is also valid for
            //   self.values.
            self.values.get_unchecked_raw_mut(idx)
        }
        // SAFETY(invariants):
        // - This function has no effect on the invariants.
    }

    /// Retrieves a reference to the value referenced by a `Pos<InUse>`.
    ///
    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    #[inline]
    pub unsafe fn get_unchecked(&self, pos: &Pos<InUse>) -> &V {
        unsafe {
            // SAFETY:
            // - The requirements are forwarded to the caller.
            // - By the invariants, any Pos<InUse> valid for this object is also valid for
            //   self.values.
            self.values.get_unchecked(pos)
        }
        // SAFETY(invariants):
        // - This function has no effect on the invariants.
    }

    /// Retrieves a mutable reference to the value referenced by a `Pos<InUse>`.
    ///
    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, pos: &Pos<InUse>) -> &mut V {
        unsafe {
            // SAFETY:
            // - The requirements are forwarded to the caller.
            // - By the invariants, any Pos<InUse> valid for this object is also valid for
            //   self.values.
            self.values.get_unchecked_mut(pos)
        }
        // SAFETY(invariants):
        // - This function has no effect on the invariants.
    }

    /// Retrieves mutable references to value referenced by `Pos<InUse>`.
    ///
    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    pub unsafe fn get_many_unchecked_mut<'s, T, U, F, G, const N: usize>(
        &'s mut self,
        pos: [Option<T>; N],
        f: F,
        g: G,
    ) -> [Option<U>; N]
    where
        F: for<'a> FnMut(&'a mut T) -> &'a mut Pos<InUse>,
        G: FnMut(T, &'s mut V) -> U,
    {
        unsafe {
            // SAFETY:
            // - By the invariants, any pos valid for this object is also valid for self.values.
            self.values.get_many_unchecked_mut(pos, f, g)
        }
        // SAFETY(invariants):
        // - This function has no effect on the invariants.
    }

    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    #[inline]
    pub unsafe fn take_unchecked(&mut self, pos: Pos<InUse>) -> V {
        let (value, pos) = unsafe {
            // SAFETY:
            // - The requirements are forwarded to the caller.
            // - By the invariants, any Pos<InUse> valid for this object is also valid for
            //   self.values.
            self.values.take_unchecked(pos)
        };
        self.free_list.push(pos);
        value
        // SAFETY(invariants):
        // - The Pos<Free> returned by self.values is valid and therefore pushing it onte
        //   self.free_list is valid.
    }

    /// Creates pointer-based access API for the vector.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn raw_access(&mut self) -> PosVecRawAccess<'_, V> {
        self.values.raw_access()
    }
}

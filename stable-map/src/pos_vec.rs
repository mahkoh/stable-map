use {
    alloc::vec::Vec,
    core::{marker::PhantomData, ptr},
    pos::{Free, InUse, Pos, Stored},
    tag::Tag,
};

pub mod pos;
mod tag;
#[cfg(test)]
mod tests;

#[derive(Debug)]
struct PositionedValue<V> {
    pos: Pos<Stored>,
    value: V,
}

/// A sparse vector with unchecked access.
///
/// Objects of this type return `Pos<InUse>` and `Pos<Free>` to the caller. These `Pos`
/// can be passed back into **this** object to perform unchecked access to these
/// positions.
///
/// Some operations cause some of these returned `Pos` to become invalid. When a `Pos`
/// becomes invalid, it must no longer be passed into this object.
//
// We maintain the following invariants:
//
// - The Pos<Stored> stored in each entry is set to its index in the vector.
// - Each returned, valid Pos<InUse> corresponds to a stored Pos<Stored>.
// - Each returned, valid Pos<Free> corresponds to an entry containing None.
// - Each returned, valid Pos has the same tag as self.tag.
//
// The following requirements are implicit:
//
// - Each exposed Pos has a unique index. This follows from all of them having the same
//   tag and the corresponding requirement of Pos.
//
// SAFETY: Each mutating function must document how it upholds these invariants.
#[derive(Debug)]
pub struct PosVec<V> {
    tag: Tag,
    values: Vec<Option<PositionedValue<V>>>,
}

pub struct PosVecRawAccess<'a, V> {
    #[cfg(test)]
    tag: Tag,
    values: *mut Option<PositionedValue<V>>,
    _phantom: PhantomData<&'a mut PosVec<V>>,
}

impl<V> PosVec<V> {
    /// Creates a new vector with the requested capacity.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tag: Tag::next(),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Returns the length of the vector.
    #[allow(clippy::len_without_is_empty)]
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns the capacity of the vector.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Reserves space for `additional` additional elements in the vector.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
    }

    /// Reduces the capacity of the vector to its length.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn shrink_to_fit(&mut self) {
        self.values.shrink_to_fit()
    }

    /// Creates a new `Pos<Free>`.
    #[cfg_attr(feature = "inline-more", inline)]
    pub(crate) fn create_pos(&mut self) -> Pos<Free> {
        let pos = unsafe {
            // SAFETY:
            // - Since the index we are using is the length of the vector (and therefore
            //   not valid), the invariants that hold at the start of this function
            //   guarantee that there is no returned, valid Pos with this index.
            Pos::new(self.tag, self.values.len())
        };
        self.values.push(None);
        pos
        // SAFETY(invariants):
        // - The Pos<Free> corresponds to the last element in self.values
        //   and that value is None.
        // - The tag is self.tag.
    }

    /// Stores a value in a `Pos<Free>`.
    ///
    /// # Safety
    ///
    /// - The `Pos<Free>` must have been returned by this object and must be valid.
    #[cfg_attr(feature = "inline-more", inline)]
    pub(crate) unsafe fn store(&mut self, pos: Pos<Free>, value: V) -> Pos<InUse> {
        #[cfg(test)]
        assert_eq!(pos.tag(), self.tag);
        let idx = pos.get();
        let (pos, stored) = pos.activate();
        let opt = unsafe {
            // SAFETY:
            // - By the invariants, the position points to a None in the vector.
            self.values.get_unchecked_mut(idx)
        };
        #[cfg(test)]
        assert!(opt.is_none());
        unsafe {
            // SAFETY:
            // - opt is a reference so this is always safe.
            // NOTE:
            // - we do this to avoid running the drop check for the old value.
            ptr::write(opt, Some(PositionedValue { pos: stored, value }));
        }
        pos
        // SAFETY(invariants):
        // - The Pos<Stored> refers to its index since the first unsafe block accesses
        //   that index.
        // - The Pos<InUse> refers to the same index and we just wrote `Some` to
        //   it.
        // - The tag of the Pos<InUse> is the tag of the input Pos<Free>. By the
        //   invariants that held before this function was called, that tag must be
        //   self.tag.
    }

    /// Removes unused slots in this PosVec.
    ///
    /// `smallest_free` must return the smallest `Pos<Free>` returned by this object.
    ///
    /// # Safety
    ///
    /// - `smallest_free` must return valid `Pos<Free>` returned by this object.
    /// - Immediately after this function returns, all previously returned `Pos<Free>`
    ///   become invalid. The caller must drop them before calling back into this object.
    #[cfg_attr(feature = "inline-more", inline)]
    pub(crate) unsafe fn compact<F>(&mut self, mut smallest_free: F)
    where
        F: FnMut() -> Option<Pos<Free>>,
    {
        // SAFETY(invariants):
        // - Note that the callback `smallest_free` cannot change self.tag since self.tag
        //   is only changed by self.clean which requires a `&mut` reference. Therefore
        //   we do not need to discuss that invariant.
        // - After this function returns, all Pos<Free> become invalid. Therefore we do
        //   not need to discuss that invariant.
        // - We never drop any Pos<Stored>, therefore it is clear that the invariant that
        //   Pos<InUse> corresponds to a Pos<Stored> continues to hold.
        // - The invariant that Pos<Stored> refers to its index in the vector is discussed
        //   below whenever we modify a position or the vector.
        'outer: while let Some(free) = smallest_free() {
            #[cfg(test)]
            assert_eq!(free.tag(), self.tag);
            // SAFETY(invariants):
            // - If this value is, None, it cannot be referred to by a Pos<InUse>.
            // - Otherwise we restore the Pos<Stored> invariant in the two branches below.
            while let Some(value) = self.values.pop() {
                if let Some(mut entry) = value {
                    if free.get() < self.values.len() {
                        let idx = unsafe {
                            // SAFETY:
                            // - By the invariants, entry.pos has the tag self.tag.
                            // - By the requirements of this method, free is a valid
                            //   Pos<Free> returned by this object. By the invariants,
                            //   that Pos<Free> has the tag self.tag.
                            entry.pos.set(free)
                        };
                        let opt = unsafe {
                            // SAFETY:
                            // - We just checked that free.get() < self.values.len().
                            self.values.get_unchecked_mut(idx)
                        };
                        #[cfg(test)]
                        assert!(opt.is_none());
                        unsafe {
                            // SAFETY:
                            // - opt is a reference, therefore ptr::write is safe.
                            // NOTE:
                            // - we do this to avoid running drop checks for *opt.
                            // SAFETY(invariants):
                            // - opt refers to the idx'th element of self.values and we
                            //   just set entry.pos to that value.
                            ptr::write(opt, Some(entry));
                        }
                        break;
                    } else {
                        // SAFETY(invariants):
                        // - We just popped this entry from the end of the vector,
                        //   therefore pushing it back restores the invariant.
                        self.values.push(Some(entry));
                        break 'outer;
                    }
                }
            }
        }
        #[cfg(test)]
        for e in &self.values {
            assert!(e.is_some());
        }
    }

    /// Removes all objects from this vector.
    ///
    /// This invalidates all `Pos<InUse>` and `Pos<Free>` previously returned by this
    /// object.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn clear(&mut self) {
        self.tag = Tag::next();
        self.values.clear();
        // SAFETY(invariants):
        // - There no longer are any valid `Pos`, therefore all invariants are trivially
        //   satisfied.
    }

    /// Retrieves a reference to a value stored at a specific index in the vector.
    ///
    /// Note that, unlike the functions taking `Pos<InUse>`, which value is returned is
    /// affected by calls to `compact`.
    #[inline]
    pub fn get(&self, pos: usize) -> Option<&V> {
        self.values
            .get(pos)
            .and_then(|v| v.as_ref())
            .map(|v| &v.value)
    }

    /// Retrieves a mutable reference to a value stored at a specific index in the vector.
    ///
    /// Note that, unlike the functions taking `Pos<InUse>`, which value is returned is
    /// affected by calls to `compact`.
    #[inline]
    pub fn get_mut(&mut self, pos: usize) -> Option<&mut V> {
        self.values
            .get_mut(pos)
            .and_then(|v| v.as_mut())
            .map(|v| &mut v.value)
    }

    /// Retrieves a reference to the value referenced by a usize.
    ///
    /// # Safety
    ///
    /// There must be a `Pos<InUse>` with the same index that could be used to call
    /// `get_unchecked`.
    #[inline]
    pub unsafe fn get_unchecked_raw(&self, idx: usize) -> &V {
        let value_opt = unsafe {
            // SAFETY:
            // - This code is identical to get_unchecked and by the requirements of this
            //   function, the safety of that function proves the safety of this function.
            self.values.get_unchecked(idx)
        };
        unsafe {
            // SAFETY:
            // - This code is identical to get_unchecked and by the requirements of this
            //   function, the safety of that function proves the safety of this function.
            &value_opt.as_ref().unwrap_unchecked().value
        }
    }

    /// Retrieves a mutable reference to the value referenced by a usize.
    ///
    /// # Safety
    ///
    /// There must be a `Pos<InUse>` with the same index that could be used to call
    /// `get_unchecked_mut`.
    #[inline]
    pub unsafe fn get_unchecked_raw_mut(&mut self, idx: usize) -> &mut V {
        let value_opt = unsafe {
            // SAFETY:
            // - This code is identical to get_unchecked and by the requirements of this
            //   function, the safety of that function proves the safety of this function.
            self.values.get_unchecked_mut(idx)
        };
        unsafe {
            // SAFETY:
            // - This code is identical to get_unchecked and by the requirements of this
            //   function, the safety of that function proves the safety of this function.
            &mut value_opt.as_mut().unwrap_unchecked().value
        }
        // SAFETY(invariants):
        // - exposing the `V` does not affect any invariants
    }

    /// Retrieves a reference to the value referenced by a `Pos<InUse>`.
    ///
    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    #[inline]
    pub unsafe fn get_unchecked(&self, pos: &Pos<InUse>) -> &V {
        #[cfg(test)]
        unsafe {
            assert_eq!(pos.tag_unchecked(), self.tag);
        }
        let idx = unsafe {
            // SAFETY:
            // - Since the Pos<InUse> is valid, the invariants guarantee that it
            //   corresponds to a Pos<Stored>. Therefore the allocation is still valid.
            pos.get_unchecked()
        };
        let value_opt = unsafe {
            // SAFETY:
            // - By the invariants, pos points in-bounds.
            self.values.get_unchecked(idx)
        };
        unsafe {
            // SAFETY:
            // - By the invariants, pos points to a Some value.
            &value_opt.as_ref().unwrap_unchecked().value
        }
    }

    /// Retrieves a mutable reference to the value referenced by a `Pos<InUse>`.
    ///
    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, pos: &Pos<InUse>) -> &mut V {
        #[cfg(test)]
        unsafe {
            assert_eq!(pos.tag_unchecked(), self.tag);
        }
        let idx = unsafe {
            // SAFETY:
            // - Since the Pos<InUse> is valid, the invariants guarantee that it
            //   corresponds to a Pos<Stored>. Therefore the allocation is still valid.
            pos.get_unchecked()
        };
        let value_opt = unsafe {
            // SAFETY:
            // - By the invariants, pos points in-bounds.
            self.values.get_unchecked_mut(idx)
        };
        unsafe {
            // SAFETY:
            // - By the invariants, pos points to a Some value.
            &mut value_opt.as_mut().unwrap_unchecked().value
        }
        // SAFETY(invariants):
        // - exposing the `V` does not affect any invariants
    }

    /// Retrieves mutable references to the value referenced by `Pos<InUse>`.
    ///
    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    pub unsafe fn get_many_unchecked_mut<'s, T, U, F, G, const N: usize>(
        &'s mut self,
        pos: [Option<T>; N],
        mut f: F,
        mut g: G,
    ) -> [Option<U>; N]
    where
        F: for<'a> FnMut(&'a mut T) -> &'a mut Pos<InUse>,
        G: FnMut(T, &'s mut V) -> U,
    {
        let values = self.values.as_mut_ptr();
        pos.map(|pos| {
            pos.map(|mut t| {
                let pos = f(&mut t);
                #[cfg(test)]
                unsafe {
                    assert_eq!(pos.tag_unchecked(), self.tag);
                }
                let idx = unsafe {
                    // SAFETY:
                    // - Since the Pos<InUse> is valid, the invariants guarantee that it
                    //   corresponds to a Pos<Stored>. Therefore the allocation is still
                    //   valid.
                    pos.get_unchecked()
                };
                let value_opt = unsafe {
                    // SAFETY:
                    // - By the invariants, pos points in-bounds.
                    // - Due to the form of F, each `&mut Pos<InUse>` must be owned by its
                    //   array element.
                    // - Therefore they must all be distinct.
                    // - Therefore, since their tags are all identical, the idx must all
                    //   be distinct.
                    // - Therefore, we only created references to distinct elements of
                    //   self.values.
                    &mut *values.add(idx)
                };
                let value = unsafe {
                    // SAFETY:
                    // - By the invariants, pos points to a Some value.
                    &mut value_opt.as_mut().unwrap_unchecked().value
                };
                g(t, value)
            })
        })
        // SAFETY(invariants):
        // - exposing the `V` does not affect any invariants
    }

    /// Consumes a `Pos<InUse>` and returns the value referenced by it.
    ///
    /// # Safety
    ///
    /// The `Pos<InUse>` must be valid and must have been returned by this object.
    #[inline]
    pub(crate) unsafe fn take_unchecked(&mut self, pos: Pos<InUse>) -> (V, Pos<Free>) {
        #[cfg(test)]
        unsafe {
            assert_eq!(pos.tag_unchecked(), self.tag);
        }
        let idx = unsafe {
            // SAFETY:
            // - Since the Pos<InUse> is valid, the invariants guarantee that it
            //   corresponds to a Pos<Stored>. Therefore the allocation is still valid.
            pos.get_unchecked()
        };
        let value = unsafe {
            // SAFETY:
            // - By the invariants, pos points in-bounds.
            self.values.get_unchecked_mut(idx)
        };
        let value = unsafe {
            // SAFETY:
            // - By the invariants, pos points to a Some value.
            value.take().unwrap_unchecked()
        };
        let pos = unsafe {
            // SAFETY:
            // - By the invariants, pos and value.pos are a pair.
            pos.deactivate(value.pos)
        };
        (value.value, pos)
        // SAFETY(invariants):
        // - We called value.take(), therefore pos refers to a None value.
        // - The tags are unaffected.
    }

    /// Creates pointer-based access API for the vector.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn raw_access(&mut self) -> PosVecRawAccess<'_, V> {
        PosVecRawAccess {
            #[cfg(test)]
            tag: self.tag,
            values: self.values.as_mut_ptr(),
            _phantom: Default::default(),
        }
    }
}

impl<'a, V> PosVecRawAccess<'a, V> {
    /// Retrieves a mutable reference to the value referenced by a `Pos<InUse>`.
    ///
    /// # Safety
    ///
    /// - The `Pos<InUse>` must be valid and must have been returned by the PosVec<V> used
    ///   to create this object.
    /// - This API must not be used to create multiple mutable references for the same
    ///   `Pos<InUse>`.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, pos: &Pos<InUse>) -> &'a mut V {
        #[cfg(test)]
        unsafe {
            assert_eq!(pos.tag_unchecked(), self.tag);
        }
        let idx = unsafe {
            // SAFETY:
            // - Since the Pos<InUse> is valid, the invariants guarantee that it
            //   corresponds to a Pos<Stored>. Therefore the allocation is still valid.
            pos.get_unchecked()
        };
        let value_opt = unsafe {
            // SAFETY:
            // - By the invariants, pos points in-bounds.
            // - By the requirements of this function, we do not create multiple mutable
            //   references to the same index.
            &mut *self.values.add(idx)
        };
        unsafe {
            // SAFETY:
            // - By the invariants, pos points to a Some value.
            &mut value_opt.as_mut().unwrap_unchecked().value
        }
        // SAFETY(invariants):
        // - exposing the `V` does not affect any invariants
    }
}

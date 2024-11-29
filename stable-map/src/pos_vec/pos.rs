use {
    crate::pos_vec::{
        pos::private::{AllocationView, Borrower, Data, Owner, TypeState},
        tag::Tag,
    },
    alloc::boxed::Box,
    core::{marker::PhantomData, mem::ManuallyDrop, ptr::NonNull},
};

/// A position in a vector.
///
/// There are three versions of this:
///
/// - `Pos<Free>`: An unoccupied position in a vector.
/// - `Pos<InUse>` and `Pos<Stored>`: An occupied position in a vector, always occur as a
///    pair.
///
/// Each `Pos` contains a pointer to an allocated `usize`. A `Pos<InUse>` and
/// `Pos<Stored>` point to the same allocation. `Pos<Free>` and `Pos<Stored>` own the
/// allocation, dropping them frees it. `Pos<InUse>` must not be used after the
/// `Pos<Stored>` has been dropped since the pointer is dangling.
///
/// `Pos<Free>` can be converted to a `Pos<InUse>`, `Pos<Stored>` pair and vice versa.
#[derive(Debug)]
pub struct Pos<T: TypeState> {
    data: NonNull<Data>,
    _phantom: PhantomData<T>,
}

mod private {
    use crate::pos_vec::tag::Tag;

    pub struct Data {
        #[cfg_attr(not(test), expect(dead_code))]
        pub tag: Tag,
        pub pos: usize,
    }

    pub trait TypeState {
        type AllocationView: AllocationView;
    }

    pub struct Owner;

    pub struct Borrower;

    pub trait AllocationView {
        const OWNER: bool;
    }

    impl AllocationView for Owner {
        const OWNER: bool = true;
    }

    impl AllocationView for Borrower {
        const OWNER: bool = false;
    }
}

#[derive(Debug)]
pub(crate) struct InUse;

#[derive(Debug)]
pub(crate) struct Free;

#[derive(Debug)]
pub(super) struct Stored;

impl TypeState for InUse {
    type AllocationView = Borrower;
}

impl TypeState for Free {
    type AllocationView = Owner;
}

impl TypeState for Stored {
    type AllocationView = Owner;
}

impl<T: TypeState> Drop for Pos<T> {
    fn drop(&mut self) {
        if <T::AllocationView as AllocationView>::OWNER {
            unsafe {
                let _ = Box::from_raw(self.data.as_ptr());
            }
        }
    }
}

impl Pos<InUse> {
    /// # Safety
    ///
    /// `self` and `stored` must be a pair returned by [Pos<Free>::activate].
    pub(super) unsafe fn deactivate(self, stored: Pos<Stored>) -> Pos<Free> {
        #[cfg(test)]
        assert_eq!(self.data, stored.data);
        let _ = ManuallyDrop::new(stored);
        Pos {
            data: self.data,
            _phantom: PhantomData,
        }
    }
}

impl Pos<Free> {
    /// # Safety
    ///
    /// For each `(tag, pos)` there must be at most one `Pos<Free>` or `Pos<Stored>`.
    pub unsafe fn new(tag: Tag, pos: usize) -> Self {
        Self {
            data: Box::leak(Box::new(Data { tag, pos })).into(),
            _phantom: PhantomData,
        }
    }

    /// Converts this object to a `Pos<InUse>`, `Pos<Stored>` pair.
    pub(super) fn activate(self) -> (Pos<InUse>, Pos<Stored>) {
        let slf = ManuallyDrop::new(self);
        let active = Pos {
            data: slf.data,
            _phantom: PhantomData,
        };
        let borrow = Pos {
            data: slf.data,
            _phantom: PhantomData,
        };
        (active, borrow)
    }
}

impl Pos<Stored> {
    /// Changes the index of this object and the corresponding `Pos<InUse>` to the index
    /// of `pos`.
    ///
    /// The invariant that `(tag, pos)` are unique is automatically enforced since the
    /// `Pos<Free>` is consumed.
    ///
    /// Returns `pos.get()`.
    ///
    /// # Safety
    ///
    /// `self` and `pos` must have the same tag.
    pub(crate) unsafe fn set(&mut self, pos: Pos<Free>) -> usize {
        #[cfg(test)]
        assert_eq!(self.tag(), pos.tag());
        let idx = pos.get();
        unsafe {
            // SAFETY:
            // - Pos<Stored> owns the allocation. Therefore the pointer is still valid.
            self.data.as_mut().pos = idx;
        }
        idx
    }
}

impl<T: TypeState<AllocationView = Owner>> Pos<T> {
    #[cfg(test)]
    pub(super) fn tag(&self) -> Tag {
        unsafe {
            // SAFETY: This Pos owns the allocation, so the pointer is valid.
            self.data.as_ref().tag
        }
    }

    pub(super) fn get(&self) -> usize {
        unsafe {
            // SAFETY: This Pos owns the allocation, so the pointer is valid.
            self.data.as_ref().pos
        }
    }
}

impl<T: TypeState<AllocationView = Borrower>> Pos<T> {
    /// # Safety
    ///
    /// The allocation pointed to by this Pos must still be valid.
    pub(crate) unsafe fn get_unchecked(&self) -> usize {
        unsafe {
            // SAFETY:
            // - The requirement is forwarded to the caller.
            self.data.as_ref().pos
        }
    }

    /// # Safety
    ///
    /// The allocation pointed to by this Pos must still be valid.
    #[cfg(test)]
    pub(super) unsafe fn tag_unchecked(&self) -> Tag {
        unsafe {
            // SAFETY:
            // - The requirement is forwarded to the caller.
            self.data.as_ref().tag
        }
    }
}

impl PartialEq for Pos<Free> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl PartialOrd for Pos<Free> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Pos<Free> {}

impl Ord for Pos<Free> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

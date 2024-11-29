#[cfg(test)]
type Data = usize;

#[cfg(not(test))]
type Data = ();

/// A unique tag.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Tag(Data);

impl Tag {
    /// Returns a new, unique tag.
    pub fn next() -> Self {
        #[cfg(test)]
        {
            use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            Self(NEXT.fetch_add(1, Relaxed))
        }
        #[cfg(not(test))]
        {
            Self(())
        }
    }
}

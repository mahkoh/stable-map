//! A hash map with temporarily-stable indices.
//!
//! This crate provides a hash map where each key is associated with an index. This index
//! remains stable unless the user explicitly compacts the map. This allows for concurrent
//! iteration over and modification of the map.

#![no_std]
extern crate alloc;

mod clone;
mod debug;
mod default;
mod drain;
mod entry;
mod eq;
mod extend;
mod from;
mod from_iterator;
mod index;
mod into_iter;
mod into_keys;
mod into_values;
mod iter;
mod iter_mut;
mod keys;
mod linear_storage;
mod map;
mod occupied_error;
mod pos_vec;
mod send_sync;
#[cfg(feature = "serde")]
mod serialize;
mod values;
mod values_mut;

pub use {
    drain::Drain,
    entry::{Entry, EntryRef, OccupiedEntry, VacantEntry, VacantEntryRef},
    into_iter::IntoIter,
    into_keys::IntoKeys,
    into_values::IntoValues,
    iter::Iter,
    iter_mut::IterMut,
    keys::Keys,
    map::StableMap,
    occupied_error::OccupiedError,
    values::Values,
    values_mut::ValuesMut,
};

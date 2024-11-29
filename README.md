# stable-map

[![crates.io](https://img.shields.io/crates/v/stable-map.svg)](http://crates.io/crates/stable-map)
[![docs.rs](https://docs.rs/stable-map/badge.svg)](http://docs.rs/stable-map)

This crate provides a hash map where each key is associated with an index. This index
remains stable unless the user explicitly compacts the map. This allows for concurrent
iteration over and modification of the map.

## Example

Consider a service that allows clients to register callbacks:

```rust
use {
    parking_lot::Mutex,
    stable_map::StableMap,
    std::sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Arc,
    },
};

pub struct Service {
    next_callback_id: AtomicUsize,
    callbacks: Mutex<StableMap<CallbackId, Arc<dyn Callback>>>,
}

pub trait Callback {
    fn run(&self);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct CallbackId(usize);

impl Service {
    pub fn register_callback(&self, callback: Arc<dyn Callback>) -> CallbackId {
        let id = CallbackId(self.next_callback_id.fetch_add(1, Relaxed));
        self.callbacks.lock().insert(id, callback);
        id
    }

    pub fn unregister_callback(&self, id: CallbackId) {
        self.callbacks.lock().remove(&id);
    }

    fn execute_callbacks(&self) {
        let mut callbacks = self.callbacks.lock();
        for i in 0..callbacks.index_len() {
            if let Some(callback) = callbacks.get_by_index(i).cloned() {
                // Drop the mutex so that the callback can itself call
                // register_callback or unregister_callback.
                drop(callbacks);
                // Run the callback.
                callback.run();
                // Re-acquire the mutex.
                callbacks = self.callbacks.lock();
            }
        }
        // Compact the map so that index_len does not grow much larger than the actual
        // size of the map.
        callbacks.compact();
    }
}
```

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.

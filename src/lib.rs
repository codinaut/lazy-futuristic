#![warn(missing_docs)]
//! Initialize variables lazily.
//! Instead of requiring a closure to initialize the value, caller can acts based on returned `ValueOrSetter`.
//!
//! # Example
//! ```
//! # tokio::runtime::Runtime::new().unwrap().block_on (async {
//! use lazy_futuristic::{Lazy, ValueOrSetter};
//!
//! let lazy_number: Lazy<i32> = Lazy::new();
//! let number = match lazy_number.get_or_set().await {
//!     ValueOrSetter::Value(value) => value,
//!     ValueOrSetter::Setter(setter) => setter.set(10),
//! };
//!
//! assert_eq!(*number, 10);
//! # });
//! ```

use futures::lock::{Mutex, MutexGuard};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// Indicates the value or needs to set the value.
#[derive(Debug)]
pub enum ValueOrSetter<'l, T> {
    /// Initialization has been done and value is returned.
    Value(&'l T),

    /// Initialization hasn't been done yet and caller has chance to initialize the value.
    Setter(Setter<'l, T>),
}

/// Setter of Lazy objects which also acts as Mutex guard.
#[derive(Debug)]
pub struct Setter<'l, T> {
    parent: &'l Lazy<T>,
    guard: MutexGuard<'l, ()>,
}

impl<'l, T> Setter<'l, T> {
    /// Set the value.
    ///
    /// Returns a reference of the stored value.
    pub fn set(&self, value: T) -> &'l T {
        unsafe {
            *self.parent.value.get() = Some(value);
            self.parent.is_initialized.store(true, Ordering::Release);
        };
        self.parent.extract()
    }
}

/// Lazy variable.
#[derive(Debug)]
pub struct Lazy<T> {
    is_initialized: AtomicBool,
    lock: Mutex<()>,
    value: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for Lazy<T> where T: Sync {}

impl<T> Default for Lazy<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Lazy<T> {
    /// Create a new lazy variable.
    pub fn new() -> Self {
        Self {
            is_initialized: AtomicBool::new(false),
            lock: Mutex::new(()),
            value: UnsafeCell::new(None),
        }
    }

    fn extract(&self) -> &T {
        if let Some(value) = unsafe { self.value.get().as_ref().unwrap() } {
            value
        } else {
            unreachable!()
        }
    }

    /// Get the value or setter will be returned if value is not available.
    #[allow(clippy::needless_lifetimes)]
    pub async fn get_or_set<'l>(&'l self) -> ValueOrSetter<'l, T> {
        if let Some(value) = self.get() {
            return ValueOrSetter::Value(value);
        }

        let guard = self.lock.lock().await;
        if let Some(value) = self.get() {
            return ValueOrSetter::Value(value);
        }
        ValueOrSetter::Setter(Setter::<T> {
            parent: &self,
            guard,
        })
    }

    /// Get the value if available.
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized.load(Ordering::Acquire) {
            Some(self.extract())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn instantiation_works() {
        Lazy::<()>::new();
    }

    #[test]
    fn share_between_threads() {
        let lazy: Lazy<i32> = Lazy::new();
        assert!(thread::spawn(move || lazy.get() == None).join().unwrap());
    }

    #[tokio::test]
    async fn get_unset() {
        let lazy: Lazy<i32> = Lazy::new();
        assert_eq!(lazy.get(), None)
    }

    #[tokio::test]
    async fn get_or_set() {
        let lazy: Lazy<i32> = Lazy::new();
        assert!(matches!(lazy.get_or_set().await, ValueOrSetter::Setter(_)));
    }

    #[tokio::test]
    async fn get_or_set_value() {
        let lazy: Lazy<i32> = Lazy::new();
        let value = match lazy.get_or_set().await {
            ValueOrSetter::Value(value) => value,
            ValueOrSetter::Setter(setter) => setter.set(5),
        };
        assert_eq!(*value, 5)
    }

    #[tokio::test]
    async fn get_and_set_double() {
        let lazy: Lazy<i32> = Lazy::new();
        match lazy.get_or_set().await {
            ValueOrSetter::Value(_) => unreachable!(),
            ValueOrSetter::Setter(setter) => setter.set(5),
        };
        match lazy.get_or_set().await {
            ValueOrSetter::Value(_) => (),
            ValueOrSetter::Setter(_) => unreachable!(),
        };
    }
}

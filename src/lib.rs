use futures::lock::{Mutex, MutexGuard};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub enum ValueOrSetter<'l, T> {
    Value(&'l T),
    Setter(Setter<'l, T>),
}

#[derive(Debug)]
pub struct Setter<'l, T> {
    parent: &'l Lazy<T>,
    guard: MutexGuard<'l, ()>,
}

impl<'l, T> Setter<'l, T> {
    pub fn set(&self, value: T) -> &'l T {
        unsafe {
            *self.parent.value.get() = Some(value);
            self.parent.is_initialized.store(true, Ordering::Release);
        };
        self.parent.extract()
    }
}

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

    #[allow(clippy::needless_lifetimes)]
    pub async fn get_or_set<'l>(&'l self) -> ValueOrSetter<'l, T> {
        if let Some(value) = self.get() {
            return ValueOrSetter::Value(value);
        }

        let guard = self.lock.lock().await;
        if self.is_initialized.load(Ordering::Relaxed) {
            ValueOrSetter::Value(self.extract())
        } else {
            ValueOrSetter::Setter(Setter::<T> {
                parent: &self,
                guard,
            })
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_initialized.load(Ordering::Relaxed) {
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

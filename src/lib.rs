use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::lock::{Mutex, MutexGuard};

#[derive(Debug)]
pub enum ValueOrSetter<'l, T> {
    Value(&'l T),
    Setter(LazySetter<'l, T>),
}

#[derive(Debug)]
pub struct LazySetter<'l, T> {
    parent: &'l Lazy<T>,
    guard: MutexGuard<'l, ()>,
}

impl<'l, T> LazySetter<'l, T> {
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

impl<T> Lazy<T> {
    pub fn new() -> Self {
        Self {
            is_initialized: AtomicBool::new(false),
            lock: Mutex::new(()),
            value: UnsafeCell::new(None),
        }
    }

    fn extract(&self) -> &T {
        match unsafe { self.value.get().as_ref().unwrap() } {
            Some(value) => value,
            None => unreachable!(),
        }
    }

    pub async fn get_or_set<'l>(&'l self) -> ValueOrSetter<'l, T> {
        if self.is_initialized.load(Ordering::Relaxed) {
            return ValueOrSetter::Value(self.extract())
        }

        let guard = self.lock.lock().await;
        if self.is_initialized.load(Ordering::Relaxed) {
            ValueOrSetter::Value(self.extract())
        } else {
            ValueOrSetter::Setter(LazySetter::<T> { parent: &self, guard })
        }
    }

    pub async fn get(&self) -> Option<&T> {
        match self.get_or_set().await {
            ValueOrSetter::Value(value) => Some(value),
            ValueOrSetter::Setter(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiation_works() {
        Lazy::<()>::new();
    }

    #[tokio::test]
    async fn get_unset() {
        let lazy: Lazy<i32> = Lazy::new();
        assert_eq!(lazy.get().await, None)
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

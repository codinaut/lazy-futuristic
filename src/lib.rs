use std::cell::UnsafeCell;

#[derive(Debug)]
pub enum ValueOrSetter<'l, T> {
    Value(&'l T),
    Setter(LazySetter<'l, T>),
}

#[derive(Debug)]
pub struct LazySetter<'l, T> {
    parent: &'l Lazy<T>
}

impl<'l, T> LazySetter<'l, T> {
    pub fn set(&self, value: T) -> &'l T {
        unsafe {
            *self.parent.value.get() = Some(value);
        };
        self.parent.get().unwrap()
    }
}

#[derive(Debug)]
pub struct Lazy<T> {
    value: UnsafeCell<Option<T>>,
}

impl<T> Lazy<T> {
    pub fn new() -> Self {
        Self { value: UnsafeCell::new(None) }
    }

    pub fn get_or_set(&self) -> ValueOrSetter<T> {
        match unsafe { self.value.get().as_ref().unwrap() } {
            Some(value) => ValueOrSetter::Value(value),
            None => ValueOrSetter::Setter(LazySetter::<T> { parent: &self })
        }
    }

    pub fn get(&self) -> Option<&T> {
        match self.get_or_set() {
            ValueOrSetter::Value(value) => Some(value),
            ValueOrSetter::Setter(_) => None
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

    #[test]
    fn get_unset() {
        let lazy = Lazy::<i32>::new();
        assert_eq!(lazy.get(), None)
    }

    #[test]
    fn get_or_set() {
        let lazy = Lazy::<i32>::new();
        assert!(matches!(lazy.get_or_set(), ValueOrSetter::Setter(_)));
    }

    #[test]
    fn get_or_set_value() {
        let lazy: Lazy<i32> = Lazy::new();
        assert_eq!(
            *match lazy.get_or_set() {
                ValueOrSetter::Value(value) => value,
                ValueOrSetter::Setter(setter) => setter.set(5),
            },
            5
        )
    }
}

#[derive(Debug)]
pub enum ValueOrSetter<'v, T> {
    Value(&'v T),
    Setter(LazySetter),
}

#[derive(Debug)]
pub struct LazySetter {}

pub struct Lazy<T> {
    value: Option<T>,
}

impl<T> Lazy<T> {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn get_or_set(&self) -> ValueOrSetter<T> {
        match &self.value {
            Some(_) => todo!(),
            None => ValueOrSetter::Setter(LazySetter {}),
        }
    }

    pub fn get(&self) -> Option<&T> {
        None
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
}

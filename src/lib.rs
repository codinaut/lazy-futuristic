pub struct Lazy<T> {
    value: Option<Box<T>>
}

impl<T> Lazy<T> {
    pub fn new() -> Self {
        Self{
            value: None
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
}

pub struct Lazy {}

impl Lazy {
    pub fn new() -> Self {
        Self{}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiation_works() {
        Lazy::new();
    }
}

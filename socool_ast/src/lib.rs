#[macro_use]
extern crate serde_derive;
pub mod ast;
pub mod operations;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate itertools;
extern crate weresocool;
extern crate num_rational;
use num_rational::{
    Ratio,
    Rational,
};


fn rational_play() {
    let d = Ratio::new(1, 7);
    let e = Ratio::new(3, 2);

    println!("{}", d + e);
    println!("{}", d * e);
    println!("{}", d / e);
    println!("{}", d - e);
}

fn main() {
    rational_play()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_render() {}
}

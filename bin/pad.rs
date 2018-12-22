extern crate weresocool;

#[derive(Debug)]
struct Op {
    a: i32,
}

fn main() {
    let input = vec![Op { a: 1 }, Op { a: 3 }];
    let modulator = vec![Op { a: 2 }, Op { a: 5 }, Op { a: 4 }];

    println!("{:?}", input);
}

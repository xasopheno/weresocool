extern crate lalrpop;

fn main() {
    println!("Building!");

    lalrpop::process_root().unwrap();
}

use failure::Fail;
use rand::{thread_rng, Rng};
use weresocool_error::Error;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
}

#[allow(unused_variables)]
fn run() -> Result<(), Error> {
    let mut rng = thread_rng();
    let n: u32 = rng.gen_range(0, 10);
    println!("{}", n);
    Ok(())
}

use failure::Fail;
use rand::{rngs::StdRng, Rng, SeedableRng};
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
    let mut rng: StdRng = SeedableRng::seed_from_u64(0);
    for i in 0..10 {
        let n: usize = rng.gen_range(0, 10);
        println!("{}", n);
    }
    //rng.reseed(&[5, 6, 7, 8]);
    Ok(())
}

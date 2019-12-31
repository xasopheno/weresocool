use error::Error;
use failure::Fail;
use weresocool::control::{setup_control, StateInterface};

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
    let state = setup_control();
    loop {
        let shared = state.get();
        //dbg!(shared);
    }
    Ok(())
}

use error::Error;
use failure::Fail;
//use weresocool::{
//generation::{filename_to_render, RenderReturn, RenderType},
//portaudio::output::output_setup,
//};

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
    Ok(())
}

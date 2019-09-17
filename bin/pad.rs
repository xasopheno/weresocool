use weresocool::{
    examples::documentation,
    generation::{filename_to_render, RenderReturn, RenderType, TimedOp},
    portaudio::live::{live_setup, State, Voice},
    instrument::{Basis, Oscillator},
    ui::{get_args, no_file_name, were_so_cool_logo},
    settings::{default_settings, Settings},
};
use socool_ast::{PointOp, NormalForm};

use error::Error;
use failure::Fail;


fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause); }
        }
    }
}


fn make_state(nf: NormalForm, basis: Basis) -> State {
    let settings = default_settings();
    let voices: Vec<Voice> = nf.operations.iter().map(|events| {
            Voice {
                events: events.to_vec(),
                remainder: None,
                oscillator: Oscillator::init(&settings),
                index: 0,
            }
    }).collect();
         
    State {
        voices, 
        basis
    } 
}

fn run() -> Result<(), Error> {
    let args = get_args();

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    let (normal_form, basis, table) = match filename_to_render(filename.unwrap(), RenderType::NfBasisAndTable)? {
        RenderReturn::NfAndBasis(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };


    println!("\nGenerating Composition ");
    let mut live_stream = live_setup(normal_form.operations, basis.clone())?;
    live_stream.start()?;

    while let true = live_stream.is_active()? {}

    live_stream.stop()?;
    Ok(())
}

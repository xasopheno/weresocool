use weresocool::{
    examples::documentation,
    generation::{filename_to_render, RenderReturn, RenderType, TimedOp, composition_to_vec_timed_op},
    portaudio::live::{live_setup, State},
    instrument::{Basis, Oscillator},
    ui::{get_args, no_file_name, were_so_cool_logo}, settings::{default_settings, Settings},
};
use socool_ast::{PointOp, NormalForm};

use error::Error;
use failure::Fail;
use num_rational::Rational64;


fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause); }
        }
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

    let (vec_timed_op, n_voices) = composition_to_vec_timed_op(&normal_form, &table);
    let mut state = State {
        ops: vec_timed_op,
        basis,
        n_voices,
        time: Rational64::new(0, 1),
        index: 0,
    };

    dbg!(state.get_batch());
    //let mut live_stream = live_setup(normal_form.operations, basis.clone())?;
    //live_stream.start()?;

    //while let true = live_stream.is_active()? {}

    //live_stream.stop()?;
    Ok(())
}

use socool_ast::{NormalForm, PointOp};
use weresocool::{
    examples::documentation,
    generation::{
        composition_to_vec_timed_op, filename_to_render, RenderReturn, RenderType, TimedOp,
    },
    instrument::{Basis, Oscillator},
    portaudio::live::{live_setup, LiveState},
    settings::{default_settings, Settings},
    ui::{get_args, no_file_name, were_so_cool_logo},
};

use error::Error;
use failure::Fail;
use num_rational::Rational64;

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

fn run() -> Result<(), Error> {
    //let args = get_args();

    //let filename = args.value_of("filename");
    //match filename {
        //Some(_filename) => {}
        //_ => no_file_name(),
    //}

    //let (normal_form, basis, table) =
        //match filename_to_render(filename.unwrap(), RenderType::NfBasisAndTable)? {
            //RenderReturn::NfAndBasis(nf, basis, table) => (nf, basis, table),
            //_ => panic!("Error. Unable to generate NormalForm"),
        //};

    //let (vec_timed_op, n_voices) = composition_to_vec_timed_op(&normal_form, &table);
    //let settings = default_settings();
    //let mut live_state = LiveState::new(vec_timed_op, n_voices, basis, &settings);

    //live_state.render_batch();
    //let mut live_stream = live_setup(live_state)?;
    //live_stream.start()?;

    //while let true = live_stream.is_active()? {}

    //live_stream.stop()?;
    //Ok(())
}

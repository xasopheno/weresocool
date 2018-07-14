extern crate portaudio;
extern crate weresocool;
use portaudio as pa;
use weresocool::oscillator::{Oscillator, Gain};
use weresocool::ratios::{R, StereoRatios, simple_ratios};
use weresocool::portaudio_output_setup::setup_portaudio_output;
use weresocool::settings::{get_default_app_settings, Settings};
use weresocool::event::{Event, Phrase, Mutate};

fn main() {
    let (l_ratios, r_ratios) = simple_ratios();
    let sr = StereoRatios {
        l_ratios,
        r_ratios
    };

    let mut e = Event::new(200.0, sr.clone(), 1.0, 1.0);
    let e = e.transpose(1.5, 0.0);

    let events = vec![
        Event::new(200.0, sr.clone(), 1.0, 1.0),
        Event::new(250.0, sr.clone(), 1.0, 1.0),
    ];

    let phrase = Phrase {
        events
    };

    println!("{:?}", phrase);
}

//fn main() {
//    match run() {
//        Ok(_) => {}
//        e => {
//            eprintln!("Failed with the following error: {:?}", e);
//        }
//    }
//}

fn run() -> Result<(), pa::Error> {
    println!("{}", "\n ***** Rust DSP __!Now In Stereo!__ ****** \n ");

    let r_ratios = vec![
        R::atio(11, 8, 0.0, 1.0),
        R::atio(1, 1, -3.0, 1.0),
        R::atio(1, 2, -0.0, 0.5),
        R::atio(1, 2, 0.5, 0.5),
    ];

    let l_ratios = vec![
        R::atio(15, 8, 0.0, 1.0),
        R::atio(15, 8, 6.0, 1.0),
        R::atio(1, 1, 0.0, 1.0),
        R::atio(1, 1, -2.0, 1.0),
    ];

    let settings: Settings = get_default_app_settings();
    let pa = pa::PortAudio::new()?;

    let mut output_stream = setup_portaudio_output(&pa)?;
    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;
    Ok(())
}

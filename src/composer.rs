extern crate portaudio;
extern crate weresocool;
use portaudio as pa;
use weresocool::portaudio_output_setup::setup_portaudio_output;
use weresocool::settings::{get_default_app_settings, Settings};

//fn main() {
//    let events = vec![
//        Event::new(200.0, simple_ratios(), 1.0, 1.0).mut_gain(0.9, 0.0),
//        Event::new(200.0, simple_ratios(), 1.0, 1.0).transpose(3.0 / 2.0, 0.0),
//        Event::new(250.0, simple_ratios(), 1.0, 1.0)
//            .mut_ratios(mono_ratios())
//            .transpose(5.0 / 4.0, 0.0)
//            .mut_length(2.0, 1.0),
//    ];
//
//    let mut phrase1 = Phrase { events };
//    println!("{:?}", phrase1);
//
//    let phrase2 = phrase1
//        .mut_ratios(mono_ratios())
//        .transpose(5.0 / 4.0, 0.0)
//        .mut_gain(0.9, 0.0)
//        .mut_length(2.0, 1.0);
//
//    println!("{:?}", phrase2);
//}

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!("{}", "\n ***** Rust DSP __!Now In Stereo!__ ****** \n ");

    let settings: Settings = get_default_app_settings();
    let pa = pa::PortAudio::new()?;

    let mut output_stream = setup_portaudio_output(&pa)?;
    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;
    Ok(())
}

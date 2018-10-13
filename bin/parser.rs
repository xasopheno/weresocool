extern crate portaudio;
extern crate weresocool;
use portaudio as pa;
use weresocool::compositions::song_22::generate_composition;
use weresocool::portaudio_setup::output::setup_portaudio_output;
use std::env;

fn main() -> Result<(), pa::Error> {
    println!("{}", "\n  ****** WereSoCool __!Now In Stereo!__ ****** ");
    println!("{}", "*** Make cool sounds. Impress your friends ***  ");
    println!("{}", " ~~~~“Catchy tunes for your next seizure.”~~~~");


//    TODO: Implement withLengthRatioOf
//    TODO: this will take the filename as an arg, pass it to the parser
//    TODO: after a successful parsed:
//
//      TODO: Init{f, l, g, p} will be passed to Event::new(f, g, p, l)
    //    TODO: init oscillator

        //fn event() -> Event {
        //    Event::init(400.0, 0.75, 0.0, 0.5)
        //}

//      TODO: Operations
//      operations = ParseTable.get("main")

//      TODO: generate events
//      let events = operation().apply(vec![event])

        // TODO: init oscillator
        //  fn oscillator() -> Oscillator {
        //  Oscillator::init(&get_default_app_settings())
        //}

        //pub fn generate_composition() -> StereoWaveform {
        //    events().render(&mut oscillator())
        //}

    let pa = pa::PortAudio::new()?;
    let mut output_stream = setup_portsaudio_output(generate_composition, &pa)?;
    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;
    Ok(())
}

extern crate portaudio;
extern crate weresocool;
extern crate colored;
extern crate socool_parser;
use portaudio as pa;
use weresocool::portaudio_setup::output::setup_portaudio_output;
use std::env;
use colored::*;
use std::collections::HashMap;
use socool_parser::parser::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename;
    if args.len() == 2 {
        filename = &args[1];
    } else {
        println!("\n{}\n", "Forgot to pass in a filename.".red().bold());
        println!("{}", "Example:".cyan());
        println!("{}\n", "./weresocool song.socool".cyan().italic());
        panic!("Wrong number of arguments.")

    }

    let parsed = parse_file(filename);

    println!("{:?}", parsed.init);
    println!("\n Main: {:?}", parsed.table.get("main").unwrap());
}

//fn main() -> Result<(), pa::Error> {
//    println!("{}", "\n  ****** WereSoCool __!Now In Stereo!__ ****** ");
//    println!("{}", "*** Make cool sounds. Impress your friends ***  ");
//    println!("{}", " ~~~~“Catchy tunes for your next seizure.”~~~~");
//
////      TODO: Init{f, l, g, p} will be passed to Event::new(f, g, p, l)
//    //    TODO: init oscillator
//
//        //fn event() -> Event {
//        //    Event::init(400.0, 0.75, 0.0, 0.5)
//        //}
//
////      TODO: Operations
////      operations = ParseTable.get("main")
//
////      TODO: generate events
////      let events = operation().apply(vec![event])
//
//        // TODO: init oscillator
//        //  fn oscillator() -> Oscillator {
//        //  Oscillator::init(&get_default_app_settings())
//        //}
//
//        //pub fn generate_composition() -> StereoWaveform {
//        //    events().render(&mut oscillator())
//        //}
//
////    let pa = pa::PortAudio::new()?;
////    let mut output_stream = setup_portaudio_output(generate_composition, &pa)?;
////    output_stream.start()?;
////
////    while let true = output_stream.is_active()? {}
////
////    output_stream.stop()?;
//    Ok(())
//}

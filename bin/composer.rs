extern crate portaudio;
extern crate weresocool;
use portaudio as pa;
use weresocool::compositions::song_10::generate_composition;
use weresocool::portaudio_setup::output::setup_portaudio_output;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!("{}", "\n  ****** WereSoCool __!Now In Stereo!__ ****** ");
    println!("{}", "*** Make cool sounds. Impress your friends ***  ");
    println!("{}", "       )))***=== COMPOSER ===***(((  \n ");
    println!("{}", " ~~~~“Catchy tunes for your next seizure.”~~~~");

    let pa = pa::PortAudio::new()?;
    let mut output_stream = setup_portaudio_output(generate_composition, &pa)?;
    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;
    Ok(())
}

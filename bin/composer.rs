extern crate portaudio;
extern crate weresocool;
use portaudio as pa;
use weresocool::portaudio_output_setup::setup_portaudio_output;

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


    let pa = pa::PortAudio::new()?;
    let mut output_stream = setup_portaudio_output(&pa)?;
    output_stream.start()?;

    while let true = output_stream.is_active()? {}

    output_stream.stop()?;
    Ok(())
}

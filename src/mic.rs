extern crate portaudio;
extern crate weresocool;
use portaudio as pa;
use weresocool::portaudio_duplex_setup::setup_portaudio_duplex;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Failed with the following error: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!("{}", "\n ***** WereSoCool __!Now In Stereo!__ ****** \n ");

    let pa = pa::PortAudio::new()?;
    let mut duplex_stream = setup_portaudio_duplex(&pa)?;

    duplex_stream.start()?;

    while let true = duplex_stream.is_active()? {}

    duplex_stream.stop()?;
    Ok(())
}

use weresocool::{
    examples::documentation,
    generation::{
        // RenderReturn,
        RenderType,
        WavType,
    },
    interpretable::{InputType::Filename, Interpretable},
    // portaudio::output_setup,
    ui::{
        // banner,
        get_args,
        no_file_name,
        were_so_cool_logo,
    },
};
use weresocool_error::Error;

fn main() -> Result<(), Error> {
    were_so_cool_logo();
    let args = get_args();

    if args.is_present("doc") {
        documentation();
    }

    let filename = args.value_of("filename");
    match filename {
        Some(_filename) => {}
        _ => no_file_name(),
    }

    if args.is_present("print") {
        Filename(filename.unwrap()).make(RenderType::Wav(WavType::MP3 { cli: true }))?;
    } else if args.is_present("json") {
        Filename(filename.unwrap()).make(RenderType::Json4d)?;
    } else if args.is_present("csv") {
        Filename(filename.unwrap()).make(RenderType::Csv1d)?;
    } else {
        // let stereo_waveform = match Filename(filename.unwrap()).make(RenderType::StereoWaveform)? {
        // RenderReturn::StereoWaveform(sw) => sw,
        // _ => panic!("Error. Unable to return StereoWaveform"),
        // };

        // let mut output_stream = output_setup(stereo_waveform)?;
        // banner("Now Playing".to_string(), filename.unwrap().to_string());

        // output_stream.start()?;
        // while let true = output_stream.is_active()? {}
        // output_stream.stop()?;
    }

    Ok(())
}

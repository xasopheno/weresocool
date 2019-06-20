use portaudio as pa;
use weresocool::{
    examples::documentation,
    generation::{filename_to_render, RenderReturn, RenderType},
    portaudio::output_setup,
    ui::{banner, get_args, no_file_name, were_so_cool_logo},
};

fn main() -> Result<(), pa::Error> {
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
        filename_to_render(filename.unwrap(), RenderType::Wav);
    } else if args.is_present("json") {
        filename_to_render(filename.unwrap(), RenderType::Json4d);
    } else if args.is_present("csv") {
        filename_to_render(filename.unwrap(), RenderType::CSV_1D);
    } else {
        let stereo_waveform =
            match filename_to_render(filename.unwrap(), RenderType::StereoWaveform) {
                RenderReturn::StereoWaveform(sw) => sw,
                _ => panic!("Error. Unable to return StereoWaveform"),
            };

        let mut output_stream = output_setup(stereo_waveform)?;
        banner("Now Playing".to_string(), filename.unwrap().to_string());

        output_stream.start()?;
        while let true = output_stream.is_active()? {}
        output_stream.stop()?;
    }

    Ok(())
}

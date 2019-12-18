use error::Error;
use failure::Fail;
use weresocool::{
    generation::{filename_to_render, RenderReturn, RenderType},
    portaudio::output::output_setup,
};

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

#[allow(unused_variables)]
fn run() -> Result<(), Error> {
    let stereo_waveform =
        match filename_to_render("songs/fall/table.socool", RenderType::StereoWaveform)? {
            RenderReturn::StereoWaveform(sw) => sw,
            _ => panic!("Error. Unable to return StereoWaveform"),
        };
    let len = &stereo_waveform.l_buffer.len();

    let mut max_d = 0.0;

    for (i, sample) in stereo_waveform.r_buffer.clone().iter_mut().enumerate() {
        if i > 0 {
            let d = &stereo_waveform.r_buffer[i] - &stereo_waveform.r_buffer[i - 1];
            let v = &stereo_waveform.r_buffer[i - 1];
            if d.abs() > max_d {
                max_d = d.abs();
            }
        }
    }
    dbg!(&len);
    dbg!(max_d);

    //output_stream.start()?;
    //while let true = output_stream.is_active()? {}
    //output_stream.stop()?;
    Ok(())
}

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
    //let offset = 0.001;
    //for (i, sample) in stereo_waveform.l_buffer.iter_mut().enumerate() {
    //if i < len / 2 {
    //*sample -= offset;
    //}
    //}
    //for (i, sample) in stereo_waveform.r_buffer.iter_mut().enumerate() {
    //if i < len / 2 {
    //*sample -= offset;
    //}
    //}
    let mut max = 0.0;
    let mut max_i = 0;
    for (i, sample) in stereo_waveform.r_buffer.clone().iter_mut().enumerate() {
        if i > 0 {
            let d = &stereo_waveform.r_buffer[i] - &stereo_waveform.r_buffer[i - 1];
            if d.abs() > max {
                max = d.abs();
                max_i = i;
            }
        }
    }
    dbg!(&len);
    dbg!(max);
    dbg!(max_i);
    let mut output_stream = output_setup(stereo_waveform)?;

    output_stream.start()?;
    while let true = output_stream.is_active()? {}
    output_stream.stop()?;
    Ok(())
}

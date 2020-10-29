//!
//! Simple example that applies reverb to the default input device's stream and passes it straight
//! to the default output device's stream.
//!

// extern crate dsp;
// extern crate lanceverb;
// extern crate portaudio as pa;

use dsp::{Frame, Node};
use lanceverb::Reverb;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {
    const CHANNELS: usize = 2;
    const FRAMES: u32 = 128;
    const SAMPLE_HZ: f64 = 44_100.0;

    // Construct the default reverb.
    let mut verb = Reverb::new();

    // Callback used to construct the duplex sound stream.
    let callback = move |pa::DuplexStreamCallbackArgs {
                             in_buffer,
                             out_buffer,
                             ..
                         }| {
        let in_buffer: &[[f32; CHANNELS]] = dsp::slice::to_frame_slice(in_buffer).unwrap();
        let out_buffer: &mut [[f32; CHANNELS]] =
            dsp::slice::to_frame_slice_mut(out_buffer).unwrap();
        dsp::slice::equilibrium(out_buffer);

        // Clamp the slice between -1.0 and 1.0.
        dsp::slice::zip_map_in_place(out_buffer, in_buffer, |_, in_frame| {
            in_frame.map(|s| {
                if s > 1.0 {
                    1.0
                } else if s < -1.0 {
                    -1.0
                } else {
                    s
                }
            })
        });

        // Apply the reverb.
        verb.audio_requested(out_buffer, SAMPLE_HZ);

        pa::Continue
    };

    // Construct PortAudio and the stream.
    let pa = pa::PortAudio::new()?;
    let chans = CHANNELS as i32;
    let settings =
        pa.default_duplex_stream_settings::<f32, f32>(chans, chans, SAMPLE_HZ, FRAMES)?;
    let mut stream = pa.open_non_blocking_stream(settings, callback);
    stream.start()?;

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep(::std::time::Duration::from_millis(16));
    }

    Ok(())
}

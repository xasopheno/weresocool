pub mod tests {
    use crate::instrument::{
        loudness::loudness_normalization,
        oscillator::Oscillator,
        stereo_waveform::StereoWaveform,
        voice::{Voice, VoiceState},
    };
    use crate::renderable::{Offset, RenderOp};
    use crate::settings::get_test_settings;
    use socool_ast::ast::{OscType, ASR};
    pub mod voice {
        use super::*;

        #[test]
        fn test_deltas() {
            let index = 1;
            let mut voice = Voice::init(index);
            let op = RenderOp::init_fglp(200.0, (0.5, 0.5), 1.0, 0.0);

            voice.update(&op);
            let p_delta = voice.calculate_portamento_delta(10);

            assert_eq!(p_delta, 20.0);
        }

        #[test]
        fn test_generate_waveform() {
            let index = 1;
            let mut voice = Voice::init(index);
            let mut op = RenderOp::init_fglp(100.0, (0.5, 0.5), 1.0, 0.0);
            op.samples = 3;
            voice.update(&op);
            let buffer = voice.generate_waveform(&op, &Offset::identity());
            assert_eq!(
                buffer,
                [0.0, 0.00000016153178806239382, 0.000000646061573488548]
            );
        }

        #[test]
        fn test_sound_silence() {
            let mut voice = Voice::init(1);
            let op1 = RenderOp::init_fglp(100.0, (0.5, 0.5), 1.0, 0.0);
            let op2 = RenderOp::init_fglp(100.0, (0.5, 0.5), 1.0, 0.0);
            voice.update(&op1);
            let silence_to_sound = voice.silence_to_sound();

            voice.update(&op2);
            let sound_to_silence = voice.sound_to_silence();

            assert_eq!(silence_to_sound, true);
            assert_eq!(sound_to_silence, false);
        }
    }
}

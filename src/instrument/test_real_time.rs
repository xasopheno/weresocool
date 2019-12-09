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
        fn test_attack_offset() {
            todo!();
            //assert_eq!();
        }
    }
}

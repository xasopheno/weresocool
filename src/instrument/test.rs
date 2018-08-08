pub mod tests {
    use instrument::{
        loudness::loudness_normalization, oscillator::Oscillator, stereo_waveform::StereoWaveform,
        voice::{Voice, VoiceState},
    };
    use ratios::{Pan, R};
    use settings::get_test_settings;
    pub mod voice {
        use super::*;
        #[test]
        fn test_voice_init() {
            let index = 1;
            let ratio = R::atio(3, 2, 0.0, 0.6, Pan::Left);
            let voice = Voice::init(index, ratio.clone());

            let result = Voice {
                ratio,
                index,
                past: VoiceState {
                    frequency: 0.0,
                    gain: 0.0,
                },
                current: VoiceState {
                    frequency: 0.0,
                    gain: 0.0,
                },
                phase: 0.0,
            };

            assert_eq!(voice, result);
        }

        #[test]
        fn test_deltas() {
            let index = 1;
            let ratio = R::atio(1, 1, 0.0, 0.6, Pan::Left);
            let mut voice = Voice::init(index, ratio.clone());
            voice.update(200.0, 1.0);
            let g_delta = voice.calculate_gain_delta(10);
            let p_delta = voice.calculate_portamento_delta(10);

            assert_eq!(g_delta, 0.039528757);
            assert_eq!(p_delta, 20.0);
        }

        #[test]
        fn test_generate_waveform() {
            let index = 1;
            let ratio = R::atio(1, 1, 0.0, 0.5, Pan::Left);
            let mut buffer = vec![0.0; 3];
            let mut voice = Voice::init(index, ratio.clone());
            voice.update(100.0, 1.0);
            voice.generate_waveform(&mut buffer, 3, 2048.0 / 44_100.0);
            assert_eq!(buffer, [0.0, 0.022728316, 0.3263405]);
        }

        #[test]
        fn test_sound_silence() {
            let ratio = R::atio(1, 1, 0.0, 1.0, Pan::Left);
            let mut voice = Voice::init(1, ratio.clone());
            voice.update(100.0, 1.0);
            let silence_to_sound = voice.silence_to_sound();

            voice.update(0.0, 0.0);
            let sound_to_silence = voice.sound_to_silence();

            assert_eq!(silence_to_sound, true);
            assert_eq!(sound_to_silence, true);
        }
    }

    pub mod oscillator {
        use super::*;
        #[test]
        fn oscillator_init_test() {
            let osc = Oscillator::init(r![(1, 1, 0.0, 1.0, 0.0),], &get_test_settings());
            let expected = Oscillator {
                portamento_length: 10,
                settings: get_test_settings(),
                voices: vec![
                    Voice {
                        index: 0,
                        ratio: R::atio(1, 1, 0.0, 0.5, Pan::Left),
                        phase: 0.0,
                        past: VoiceState {
                            frequency: 0.0,
                            gain: 0.0,
                        },
                        current: VoiceState {
                            frequency: 0.0,
                            gain: 0.0,
                        },
                    },
                    Voice {
                        index: 1,
                        ratio: R::atio(1, 1, 0.0, 0.5, Pan::Right),
                        phase: 0.0,
                        past: VoiceState {
                            frequency: 0.0,
                            gain: 0.0,
                        },
                        current: VoiceState {
                            frequency: 0.0,
                            gain: 0.0,
                        },
                    },
                ],
            };
            assert_eq!(osc, expected);
        }

        #[test]
        fn oscillator_ratio_update_test() {
            let mut osc = Oscillator::init(r![(1, 1, 0.0, 1.0, 0.0),], &get_test_settings());
            osc.update_ratios(&r![(3, 2, 0.0, 1.0, 0.0),]);

            assert_eq!(osc.voices[0].ratio, R::atio(3, 2, 0.0, 0.5, Pan::Left));
            assert_eq!(osc.voices[1].ratio, R::atio(3, 2, 0.0, 0.5, Pan::Right))
        }
        #[test]
        fn oscillator_generate_test() {
            let mut osc = Oscillator::init(r![(1, 1, 0.0, 1.0, 0.0),], &get_test_settings());

            osc.update_freq_gain_and_ratios(200.0, 1.0, &r![(3, 2, 0.0, 1.0, 0.0)]);
            assert_eq!(osc.voices[0].past.frequency, 0.0);
            assert_eq!(osc.voices[0].past.gain, 0.0);
            assert_eq!(osc.voices[0].current.frequency, 300.0);
            assert_eq!(osc.voices[0].current.gain, 0.25805622);

            assert_eq!(osc.voices[0].past.frequency, 0.0);
            assert_eq!(osc.voices[0].past.gain, 0.0);
            assert_eq!(osc.voices[0].current.frequency, 300.0);
            assert_eq!(osc.voices[0].current.gain, 0.25805622);

            let expected = StereoWaveform {
                l_buffer: vec![0.0, 0.0073444042, 0.021999665],
                r_buffer: vec![0.0, 0.0073444042, 0.021999665],
            };
            assert_eq!(osc.generate(3), expected);
        }
    }

    pub mod loudness {
        use super::*;
        #[test]
        fn test_loudness_normalization() {
            let expected = loudness_normalization(0.0);
            let result = 0.0;
            assert_eq!(expected, result);

            let expected = loudness_normalization(10.0);
            let result = 0.0;
            assert_eq!(expected, result);

            let expected = loudness_normalization(100.0);
            let result = 1.0;
            assert_eq!(expected, result);

            let expected = loudness_normalization(250.0);
            let result = 0.5759918;
            assert_eq!(expected, result);

            let expected = loudness_normalization(500.0);
            let result = 0.3794706;
            assert_eq!(expected, result);

            let expected = loudness_normalization(1000.0);
            let result = 0.25;
            assert_eq!(expected, result);

            let expected = loudness_normalization(1500.0);
            let result = 0.19584954;
            assert_eq!(expected, result);
        }
    }
}

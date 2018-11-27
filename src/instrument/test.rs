pub mod tests {
    use event::Sound;
    use instrument::{
        loudness::loudness_normalization,
        oscillator::Oscillator,
        stereo_waveform::StereoWaveform,
        voice::{Voice, VoiceState},
    };
    use settings::get_test_settings;
    pub mod voice {
        use super::*;
        #[test]
        fn test_voice_init() {
            let index = 1;
            let voice = Voice::init(index);

            let result = Voice {
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
            let mut voice = Voice::init(index);
            voice.update(200.0, 1.0);
            let g_delta = voice.calculate_gain_delta(10);
            let p_delta = voice.calculate_portamento_delta(10);

            assert_eq!(g_delta, 0.06588126);
            assert_eq!(p_delta, 20.0);
        }

        #[test]
        fn test_generate_waveform() {
            let index = 1;
            let mut buffer = vec![0.0; 3];
            let mut voice = Voice::init(index);
            voice.update(100.0, 1.0);
            voice.generate_waveform(&mut buffer, 3, 2048.0 / 44_100.0);
            assert_eq!(buffer, [0.0, 0.045456633, 0.652681]);
        }

        #[test]
        fn test_sound_silence() {
            let mut voice = Voice::init(1);
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
            let osc = Oscillator::init(&get_test_settings());
            let expected = Oscillator {
                portamento_length: 10,
                sample_phase: 0.0,
                settings: get_test_settings(),
                voices: vec![(
                    Voice {
                        index: 0,
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
                )],
            };
            assert_eq!(osc, expected);
        }

        #[test]
        fn oscillator_update_test() {
            let mut osc = Oscillator::init(&get_test_settings());
            osc.update(vec![Sound {
                frequency: 100.0,
                gain: 1.0,
                pan: 0.5,
            }]);

            assert_eq!(osc.voices[0].0.past.frequency, 0.0);
            assert_eq!(osc.voices[0].0.past.gain, 0.0);
            assert_eq!(osc.voices[0].0.current.frequency, 100.0);
            assert_eq!(osc.voices[0].0.current.gain, 0.25);
            //
            assert_eq!(osc.voices[0].1.past.frequency, 0.0);
            assert_eq!(osc.voices[0].1.past.gain, 0.0);
            assert_eq!(osc.voices[0].1.current.frequency, 100.0);
            assert_eq!(osc.voices[0].1.current.gain, 0.75);
        }
        #[test]
        fn oscillator_generate_test() {
            let mut osc = Oscillator::init(&get_test_settings());

            osc.update(vec![Sound {
                frequency: 300.0,
                gain: 1.0,
                pan: -0.5,
            }]);

            let expected = StereoWaveform {
                l_buffer: vec![0.0, 0.011016606, 0.032999497],
                r_buffer: vec![0.0, 0.0036722021, 0.010999832],
            };
            assert_eq!(osc.generate(3.0), expected);
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

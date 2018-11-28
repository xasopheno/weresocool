//pub mod apply {
//    extern crate num_rational;
//    use event::Event;
//    use generation::parsed_to_waveform::r_to_f32;
//    use operations::helpers::helpers::vv_event_to_v_events;
//    use operations::{Apply, PointOp};
//
//    impl Apply<Event> for PointOp {
//        fn apply(&self, mut event: Event) -> Event {
//            for mut sound in event.sounds.iter_mut() {
//                    sound.frequency *= self.fm;
//                    sound.frequency += self.fa;
//                    sound.pan *= self.pm;
//                    sound.pan += self.pa;
//                    sound.gain *= self.g;
//                }
//
//            event.length += self.l;
//
//            event
//        }
//    }
//}

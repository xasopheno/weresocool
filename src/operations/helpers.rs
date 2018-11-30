//#[allow(dead_code)]
//pub mod helpers {
//    use event::Event;
//
//    pub fn vv_event_to_v_events(vv_events: &Vec<Vec<Event>>) -> Vec<Event> {
//        let mut accumulator = vec![];
//        let mut state = vv_events.clone();
//        while state.len() > 0 {
//            state.retain(|ref x| x.len() > 0);
//            fold_vv_events(&mut accumulator, &mut state);
//        }
//        accumulator
//    }
//
//    fn fold_vv_events(accumulator: &mut Vec<Event>, state: &mut Vec<Vec<Event>>) {
//        let next_length = next_length(&state);
//        let mut s = state.clone();
//        let mut events_to_join = vec![];
//        for (index, vec_event) in state.iter_mut().enumerate() {
//            if vec_event.len() > 0 {
//                let current = vec_event.remove(0);
//                if current.length > next_length {
//                    let mut remainder = current.clone();
//                    remainder.length = current.length - next_length;
//                    vec_event.insert(0, remainder)
//                }
//                events_to_join.push(current);
//            } else {
//                s.remove(index);
//            }
//        }
//
//        if events_to_join.len() > 0 {
//            let event = join_events(events_to_join, next_length);
//            if event.length > 0.005 {
//                accumulator.push(event)
//            }
//        }
//    }
//
//    fn join_events(events: Vec<Event>, length: f32) -> Event {
//        let mut sounds = vec![];
//
//        for mut event in events {
//            sounds.append(&mut event.sounds)
//        }
//
//        Event { sounds, length }
//    }
//
//    fn next_length(state: &Vec<Vec<Event>>) -> f32 {
//        let mut values = vec![];
//        for vec_event in state.iter() {
//            values.push(vec_event[0].length)
//        }
//        let min = values.iter().cloned().fold(1.0 / 0.0, f32::min);
//
//        if min.is_infinite() {
//            0.0
//        } else {
//            min
//        }
//    }
//}

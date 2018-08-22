pub mod apply {
    use operations::helpers::helpers::vv_event_to_v_events;
    use operations::{Op, Apply, GetLengthRatio};
    use event::Event;

    impl Apply for Op {
        fn apply(&self, events: Vec<Event>) -> Vec<Event> {
            let mut vec_events: Vec<Event> = vec![];
            match self {
                Op::AsIs {} => {
                    vec_events = events;
                }

                Op::Reverse {} => {
                    let mut clone = events.clone();
                    clone.reverse();
                    vec_events = clone
                }

                Op::TransposeM { m } => {
                    for event in events.iter() {
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.frequency = sound.frequency * m;
                        }
                        vec_events.push(e)
                    }
                }

                Op::TransposeA { a } => {
                    for event in events.iter() {
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.frequency = sound.frequency + a;
                        }
                        vec_events.push(e)
                    }
                }

                Op::PanA { a } => {
                    for event in events.iter() {
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.pan += a;
                        }
                        vec_events.push(e)
                    }
                }

                Op::PanM { m } => {
                    for event in events.iter() {
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.pan *= m;
                        }
                        vec_events.push(e)
                    }
                }

                Op::Length { m } => {
                    for event in events.iter() {
                        let mut e = event.clone();
                        e.length = e.length * m;
                        vec_events.push(e)
                    }
                }

                Op::Repeat { n, operations } => {
                    let mut repeat_container: Vec<Event> = vec![];

                    let sequence = Op::Sequence {
                        operations: operations.to_vec(),
                    }.apply(events);

                    for _ in 0..*n {
                        repeat_container.append(&mut sequence.clone());
                    }

                    vec_events = repeat_container;
                }

                Op::Silence { m } => {
                    for event in events.iter() {
                        let mut e = event.clone();
                        e.length *= m;
                        for sound in e.sounds.iter_mut() {
                            sound.frequency = 0.0;
                            sound.gain = 0.0;
                        }
                        vec_events.push(e)
                    }
                }

                Op::Gain { m } => {
                    for event in events.iter() {
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.gain = sound.gain * m;
                        }
                        vec_events.push(e)
                    }
                }

                Op::Compose { operations } => {
                    let mut es = events.clone();
                    for operation in operations.iter() {
                        es = operation.apply(es);
                    }
                    vec_events = es;
                }

                Op::Sequence { operations } => {
                    let mut es = events.clone();
                    let mut container = vec![];
                    for operation in operations.iter() {
                        container.push(operation.apply(es.clone()));
                    }

                    vec_events = container.iter().flat_map(|evt| evt.clone()).collect();
                }

                Op::Fit {
                    with_length_of,
                    main,
                    n,
                } => {
                    let mut es = events.clone();

                    let target_length = with_length_of.get_length_ratio();
                    let main_length = main.get_length_ratio();
                    let ratio = target_length / main_length / *n as f32;

                    let mut array_of_main = vec![];
                    for _ in 0..*n {
                        array_of_main.push(*main.clone())
                    }

                    let main_sequence = Op::Sequence {
                        operations: array_of_main,
                    };

                    let new_op = Op::Compose {
                        operations: vec![main_sequence, Op::Length { m: ratio }],
                    };

                    vec_events = new_op.apply(es);
                }

                Op::Overlay { operations } => {
                    let mut vec_vec_events: Vec<Vec<Event>> = vec![];

                    for operation in operations.iter() {
                        let es = operation.apply(events.clone());
                        vec_vec_events.push(es);
                    }

                    vec_events = vv_event_to_v_events(&vec_vec_events);
                }
            }

            vec_events
        }
    }
}
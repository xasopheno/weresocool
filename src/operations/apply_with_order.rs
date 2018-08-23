pub mod apply_with_order {
    use event::Event;
    use operations::helpers::helpers::vv_event_to_v_events;
    use operations::{Apply, ApplyWithOrder, GetLengthRatio, Op};

    impl ApplyWithOrder for Op {
        fn apply_with_order(
            &self,
            order_fn: fn(order: usize, length: usize) -> f32,
            events: Vec<Event>,
        ) -> Vec<Event> {
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
                    for (order, event) in events.iter().enumerate() {
                        let order_after_fn = order_fn(order,events.len());
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.frequency = sound.frequency * m * order_after_fn;
                        }
                        vec_events.push(e)
                    }
                }

                Op::TransposeA { a } => {
                    for (order, event) in events.iter().enumerate() {
                        let order_after_fn = order_fn(order, events.len());
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.frequency = sound.frequency + a * order_after_fn;
                        }
                        vec_events.push(e)
                    }
                }

                Op::PanM { m } => {
                    for (order, event) in events.iter().enumerate() {
                        let order_after_fn = order_fn(order, events.len());
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.pan *= m * order_after_fn;
                        }
                        vec_events.push(e)
                    }
                }

                Op::PanA { a } => {
                    for (order, event) in events.iter().enumerate() {
                        let order_after_fn = order_fn(order, events.len());
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.pan += a * order_after_fn;
                        }
                        vec_events.push(e)
                    }
                }

                Op::Length { m } => {
                    for (order, event) in events.iter().enumerate() {
                        let order_after_fn = order_fn(order, events.len());
                        let mut e = event.clone();
                        e.length = e.length * m * order_after_fn;
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
                    for (order, event) in events.iter().enumerate() {
                        let order_after_fn = order_fn(order, events.len());
                        let mut e = event.clone();
                        e.length *= m * order_after_fn;
                        for sound in e.sounds.iter_mut() {
                            sound.frequency = 0.0;
                            sound.gain = 0.0;
                        }
                        vec_events.push(e)
                    }
                }

                Op::Gain { m } => {
                    for (order, event) in events.iter().enumerate() {
                        let order_after_fn = order_fn(order, events.len());
                        let mut e = event.clone();
                        for sound in e.sounds.iter_mut() {
                            sound.gain = sound.gain * m * order_after_fn;
                        }
                        vec_events.push(e)
                    }
                }

                Op::Compose { operations } => {
                    let mut es = events.clone();
                    for operation in operations.iter() {
                        es = operation.apply_with_order(order_fn, es);
                    }
                    vec_events = es;
                }

                Op::ComposeWithOrder {
                    order_fn,
                    operations,
                } => {
                    let mut es = events.clone();
                    for operation in operations.iter() {
                        es = operation.apply_with_order(*order_fn, es);
                    }
                    vec_events = es;
                }

                Op::Sequence { operations } => {
                    let mut es = events.clone();
                    let mut container = vec![];
                    for operation in operations.iter() {
                        container.push(operation.apply_with_order(order_fn, es.clone()));
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

                    vec_events = new_op.apply_with_order(order_fn, es);
                }

                Op::Overlay { operations } => {
                    let mut vec_vec_events: Vec<Vec<Event>> = vec![];

                    for operation in operations.iter() {
                        let es = operation.apply_with_order(order_fn, events.clone());
                        vec_vec_events.push(es);
                    }

                    vec_events = vv_event_to_v_events(&vec_vec_events);
                }
            }

            vec_events
        }
    }
}

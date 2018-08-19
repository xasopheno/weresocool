use event::Event;

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    AsIs,
    Reverse,
    Pan {
        a: f32,
    },
    TransposeM {
        m: f32,
    },
    TransposeA {
        a: f32,
    },
    Silence {
        m: f32,
    },
    Length {
        m: f32,
    },
    Gain {
        m: f32,
    },
    Repeat {
        n: usize,
        operations: Vec<Op>,
    },
    Sequence {
        operations: Vec<Op>,
    },
    Compose {
        operations: Vec<Op>,
    },
    Fit {
        with_length_of: Box<Op>,
        main: Box<Op>,
        n: usize,
    },
    Overlay {
        operations: Vec<Op>,
    },
}

pub trait Operate {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
    fn get_length_ratio(&self) -> f32;
}

impl Operate for Op {
    fn get_length_ratio(&self) -> f32 {
        match self {
            Op::AsIs {}
            | Op::Reverse {}
            | Op::TransposeM { m: _ }
            | Op::TransposeA { a: _ }
            | Op::Pan { a: _ }
            | Op::Gain { m: _ } => 1.0,

            Op::Repeat { n, operations } => {
                let mut length_ratio_of_operations = 0.0;
                for operation in operations {
                    length_ratio_of_operations += operation.get_length_ratio();
                }

                length_ratio_of_operations * *n as f32
            }

            Op::Length { m } | Op::Silence { m } => *m,

            Op::Sequence { operations } => {
                let mut new_total = 0.0;
                for operation in operations {
                    new_total += operation.get_length_ratio();
                }
                new_total
            }
            Op::Compose { operations } => {
                let mut new_total = 1.0;
                for operation in operations {
                    new_total *= operation.get_length_ratio();
                }
                new_total
            }

            Op::Fit {
                with_length_of,
                main: _,
                n: _,
            } => with_length_of.get_length_ratio(),

            Op::Overlay { operations } => {
                let mut max = 0.0;
                for op in operations {
                    let next = op.get_length_ratio();
                    if next > max {
                        max = next;
                    }
                }
                max
            }
        }
    }

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

            Op::Pan { a } => {
                for event in events.iter() {
                    let mut e = event.clone();
                    for sound in e.sounds.iter_mut() {
                        sound.pan += a;
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
                let mut es = events.clone();
                let mut container: Vec<Event> = vec![];
                let mut repeat_container: Vec<Event> = vec![];

                let sequence = Op::Sequence { operations: operations.to_vec() }.apply(events);

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

fn vv_event_to_v_events(vv_events: &Vec<Vec<Event>>) -> Vec<Event> {
    let mut accumulator = vec![];
    let mut state = vv_events.clone();
    while state.len() > 0 {
        state.retain(|ref x| x.len() > 0);
        fold_vv_events(&mut accumulator, &mut state);
    }
    accumulator
}

fn fold_vv_events(accumulator: &mut Vec<Event>, state: &mut Vec<Vec<Event>>) {
    let next_length = next_length(&state);
    let mut s = state.clone();
    let mut events_to_join = vec![];
    for (index, vec_event) in state.iter_mut().enumerate() {
        if vec_event.len() > 0 {
            let current = vec_event.remove(0);
            if current.length > next_length {
                let mut remainder = current.clone();
                remainder.length = current.length - next_length;
                vec_event.insert(0, remainder)
            }
            events_to_join.push(current);
        } else {
            s.remove(index);
        }
    }

    if events_to_join.len() > 0 {
        let event = join_events(events_to_join, next_length);
        if event.length > 0.0001 {
            accumulator.push(event)
        }
    }
}

fn join_events(events: Vec<Event>, length: f32) -> Event {
    let mut sounds = vec![];

    for mut event in events {
        sounds.append(&mut event.sounds)
    }

    Event { sounds, length }
}

fn next_length(state: &Vec<Vec<Event>>) -> f32 {
    let mut values = vec![];
    for vec_event in state.iter() {
        values.push(vec_event[0].length)
    }
    let min = values.iter().cloned().fold(1.0 / 0.0, f32::min);

    if min.is_infinite() {
        0.0
    } else {
        min
    }
}

#[cfg(test)]
mod test;

use event::Event;
use ratios::R;

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    AsIs,
    Transpose {
        m: f32,
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
    Ratios {
        ratios: Vec<R>,
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
            | Op::Transpose { m: _, a: _ }
            | Op::Gain { m: _ }
            | Op::Ratios { ratios: _ } => 1.0,

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
            } => with_length_of.get_length_ratio(),
        }
    }

    fn apply(&self, events: Vec<Event>) -> Vec<Event> {
        let mut vec_events: Vec<Event> = vec![];
        match self {
            Op::AsIs {} => {
                vec_events = events;
            }

            Op::Transpose { m, a } => {
                for event in events.iter() {
                    let mut e = event.clone();
                    e.frequency = e.frequency * m + a;
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

            Op::Silence { m } => {
                for event in events.iter() {
                    let mut e = event.clone();
                    e.length *= m;
                    e.frequency = 0.0;
                    e.gain = 0.0;
                    vec_events.push(e)
                }
            }

            Op::Gain { m } => {
                for event in events.iter() {
                    let mut e = event.clone();
                    e.gain = e.gain * m;
                    vec_events.push(e)
                }
            }

            Op::Ratios { ratios } => {
                for event in events.iter() {
                    let mut es = event.clone();
                    es.ratios = ratios.clone();
                    vec_events.push(es)
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
            } => {
                let mut es = events.clone();
                let target_length = with_length_of.get_length_ratio();
                let main_length = main.get_length_ratio();
                let ratio = target_length / main_length;

                let new_op = Op::Compose {
                    operations: vec![*main.clone(), Op::Length { m: ratio }],
                };

                vec_events = new_op.apply(es);
            }
        }

        vec_events
    }
}

#[cfg(test)]
mod test;

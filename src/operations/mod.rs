use event::Event;
use ratios::R;

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    AsIs,
    Transpose { m: f32, a: f32 },
    Length { m: f32, a: f32 },
    Gain { m: f32, a: f32 },
    Ratios { ratios: Vec<R> },
    Compose { operations: Vec<Op> },
    Sequence { operations: Vec<Op> },
}

pub trait Operate {
    fn apply(&self, events: Vec<Event>) -> Vec<Event>;
}

impl Operate for Op {
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

            Op::Length { m, a } => {
                for event in events.iter() {
                    let mut e = event.clone();
                    e.length = e.length * m + a;
                    vec_events.push(e)
                }
            }

            Op::Gain { m, a } => {
                for event in events.iter() {
                    let mut e = event.clone();
                    e.gain = e.gain * m + a;
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
        }

        vec_events
    }
}

//ops.apply(event)

//thing1 = [
//    _;
//    ^ 3.0/2.0;
//    compose(
//         r[
//            (1, 2, 0.0, 0.3, 0.0)
//            (1, 2, 1.0, 0.3, 0.0)
//         ],
//         l 2.0, 0.0
//    );
//    compose(
//         r
//            (3, 2, 0.0, 0.5, 0.3)
//            (3, 2, 4.0, 0.5, -0.3)
//         l 1.0, 2.0
//         ^ 9.0/8.0
//    );
//     r[
//        (1, 2, 0.0, 0.3, 0.0)
//        (1, 2, 1.0, 0.3, 0.0)
//      ];
//]

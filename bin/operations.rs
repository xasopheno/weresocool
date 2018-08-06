extern crate weresocool;
use weresocool::ratios::R;
use weresocool::operations::{Op, Operate};
use weresocool::event::Event;

fn main() {
    let r = r![
        (5, 4, 0.0, 0.6, -1.0),
        (1, 2, -1.0, 0.6, 1.0),
        (0, 2, -1.0, 0.0, 1.0),
        (0, 2, -1.0, 0.0, 1.0),
    ];
    let e = Event::new(100.0, r, 1.0, 1.0);

    let ops = Op::Sequence {
        operations: vec![

        ]
    };

}
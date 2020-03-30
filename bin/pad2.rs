//use weresocool::ui::were_so_cool_logo;
//use std::sync::{Arc, Mutex};

use failure::Fail;
use weresocool_error::Error;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Renderer {
    new: bool,
    voices: [Voice; 2],
    buffers: [Vec<usize>; 2],
    index: usize,
}

#[derive(Clone, PartialEq, Debug)]
struct Voice {
    ops: Vec<usize>,
    read_idx: usize,
}

impl Voice {
    fn new(ops: Vec<usize>) -> Self {
        Self { ops, read_idx: 0 }
    }
}

impl Renderer {
    fn new() -> Self {
        let ops1: Vec<usize> = (0..10).into_iter().collect();
        let ops2: Vec<usize> = (10..20).into_iter().collect();
        Self {
            new: false,
            voices: [Voice::new(ops1), Voice::new(ops2)],
            buffers: [vec![], vec![]],
            index: 0,
        }
    }

    fn inc(&mut self) {
        self.index = (self.index + 1) % 2;
    }

    fn render(&mut self) {
        for _ in 0..1 {
            let voice = &mut self.voices[self.index];
            let buffer = &mut self.buffers[self.index];
            buffer.push(voice.ops[voice.read_idx]);

            if voice.read_idx >= voice.ops.len() - 1 {
                voice.read_idx = 0
            } else {
                voice.read_idx += 1
            }
        }
    }
}

fn test_inc() {
    let mut r = Renderer::new();

    r.inc();
    assert!(r.index == 1);
    r.inc();
    assert!(r.index == 0);
    r.inc();
    assert!(r.index == 1);
}

fn run() -> Result<(), Error> {
    test_inc();
    let mut r = Renderer::new();

    for i in 0..5 {
        if i == 3 {
            r.inc();
        }
        r.render();
        dbg!(&r.buffers[r.index]);
    }

    Ok(())
}

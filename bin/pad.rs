use weresocool::generation::{
    composition_to_vec_timed_op, filename_to_render, vec_timed_op_to_vec_op4d, EventType, Op4D,
    RenderReturn, RenderType,
};

struct Normalizer {
    t: MinMaxF,
    event: MinMaxU,
    event_type: EventType,
    voice: MinMaxU,
    x: MinMaxF,
    y: MinMaxF,
    z: MinMaxF,
    l: MinMaxF,
}

struct MinMaxF {
    min: f64,
    max: f64,
}

struct MinMaxU {
    min: usize,
    max: usize,
}

fn main() {
    println!("Hello Scratch Pad");
    let (normal_form, basis, table) = match filename_to_render(
        &"songs/spring/silly_day.socool".to_string(),
        RenderType::NfBasisAndTable,
    ) {
        RenderReturn::NfAndBasis(nf, basis, table) => (nf, basis, table),
        _ => panic!("Error. Unable to generate NormalForm"),
    };

    let vec_timed_op = composition_to_vec_timed_op(&normal_form, &table);
    let vec_op4d = vec_timed_op_to_vec_op4d(vec_timed_op, &basis);
}

fn get_min_max_op4d_1d(vec_op4d: Vec<Op4D>) {
    let mut max_state = Op4D {
        t: 0.0,
        event: 0,
        event_type: EventType::On,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
    };

    let mut min_state = Op4D {
        t: 0.0,
        event: 0,
        event_type: EventType::On,
        voice: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        l: 0.0,
    };

    for op in vec_op4d {
        max_state = Op4D {
            x: max_state.x.max(op.x),
            y: max_state.y.max(op.y),
            z: max_state.z.max(op.z),
            l: max_state.l.max(op.l),
            t: max_state.t.max(op.t),
            event: max_state.event.max(op.event),
            voice: max_state.voice.max(op.voice),
            event_type: EventType::On,
        };
        min_state = Op4D {
            x: min_state.x.min(op.x),
            y: min_state.y.min(op.y),
            z: min_state.z.min(op.z),
            l: min_state.l.min(op.l),
            t: min_state.t.min(op.t),
            event: min_state.event.min(op.event),
            voice: min_state.voice.min(op.voice),
            event_type: EventType::On,
        };
    }
    dbg!(max_state);
    dbg!(min_state);
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}

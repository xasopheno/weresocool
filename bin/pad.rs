use weresocool::generation::{
    composition_to_vec_timed_op, filename_to_render,
    json::{MinMax, Normalizer},
    vec_timed_op_to_vec_op4d, EventType, Op4D, RenderReturn, RenderType,
};

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
    let mut op4d_1d = vec_timed_op_to_vec_op4d(vec_timed_op, &basis);

    let normalizer = get_min_max_op4d_1d(&op4d_1d);
    let normalized_op4_1d = normalized_op4d_1d(&mut op4d_1d, normalizer);
}

fn normalized_op4d_1d(op4d_1d: &mut Vec<Op4D>, n: Normalizer) {
    op4d_1d.iter_mut().for_each(|op| {
        op.normalize(&n);
    })
}

fn get_min_max_op4d_1d(vec_op4d: &Vec<Op4D>) -> Normalizer {
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

    Normalizer {
        x: MinMax {
            min: min_state.x,
            max: max_state.x,
        },
        y: MinMax {
            min: min_state.y,
            max: max_state.y,
        },
        z: MinMax {
            min: min_state.z,
            max: max_state.z,
        },
    }
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}

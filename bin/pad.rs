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
}

#[test]
fn test_decimal_to_fraction() {
    assert!(true, true);
}

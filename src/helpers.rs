use float_cmp::ApproxEq;

pub fn cmp_vec_f32(vec1: Vec<f32>, vec2: Vec<f32>) -> bool {
    for (a, b) in vec1.iter().zip(vec2) {
        if !a.approx_eq(b, (0.0, 2)) {
            return false;
        }
    }
    true
}

pub fn cmp_vec_f64(vec1: Vec<f64>, vec2: Vec<f64>) -> bool {
    for (a, b) in vec1.iter().zip(vec2) {
        if !a.approx_eq(b, (0.0, 2)) {
            return false;
        }
    }
    true
}

pub fn cmp_f32(a: f32, b: f32) -> bool {
    a.approx_eq(b, (0.0, 2))

}

pub fn cmp_f64(a: f64, b: f64) -> bool {
    a.approx_eq(b, (0.0, 2))
}


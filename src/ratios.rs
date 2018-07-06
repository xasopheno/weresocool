#[derive(Debug)]
pub struct R {
    pub decimal: f32,
    pub offset: f32,
    pub ratio: String,
    pub gain: f32,
}

impl R {
    pub fn atio(n: usize, d: usize, offset: f32, gain: f32) -> R {
        if d == 0 {
            panic!("Denominator of a Ratio cannot be 0. Failed at R.atio({}, {}, {}, {})", n, d, offset, gain);
        }
        R {
            decimal: n as f32 / d as f32,
            offset,
            ratio: [n.to_string(), d.to_string()].join("/"),
            gain,
        }
    }
}

pub fn complicated_ratios() -> (Vec<R>, Vec<R>) {
    let l_ratios = vec![
        R::atio(23, 2, 0.0, 0.04),
        R::atio(23, 2, -0.1, 0.04),
        R::atio(19, 2, 0.0, 0.1),
        R::atio(19, 2, -0.2, 0.1),
        R::atio(15, 2, 18.0, 0.15),
        R::atio(15, 2, 0.0, 0.15),
        R::atio(10, 2, -9.0, 0.15),
        R::atio(7, 2, 1.0, 1.0),
        R::atio(7, 2, 0.0, 1.0),
        R::atio(3, 2, 3.0, 1.0),
        R::atio(12, 4, 0.0, 1.0),
        R::atio(15, 8, 0.0, 1.0),
        R::atio(15, 8, 6.0, 1.0),
        R::atio(1, 1, 0.0, 1.0),
        R::atio(1, 1, -2.0, 1.0),
        R::atio(1, 2, 0.0, 0.5),
        R::atio(1, 2, 0.5, 0.5),
        R::atio(1, 4, 1.0, 0.6),
        R::atio(1, 4, 0.0, 0.6),
    ];

    let r_ratios = vec![
        R::atio(21, 2, 0.0, 0.05),
        R::atio(21, 2, 0.2, 0.05),
        R::atio(17, 2, 0.0, 0.1),
        R::atio(17, 2, 0.3, 0.1),
        R::atio(13, 2, 0.0, 0.15),
        R::atio(13, 2, -11.0, 0.15),
        R::atio(11, 2, 5.0, 0.15),
        R::atio(11, 2, 0.0, 0.15),
        R::atio(12, 4, 0.0, 1.0),
        R::atio(9, 4, 0.0, 1.0),
        R::atio(9, 4, 6.0, 1.0),
        R::atio(5, 4, 0.0, 1.0),
        R::atio(7, 3, -3.0, 1.0),
        R::atio(11, 8, 0.0, 1.0),
        R::atio(1, 1, -3.0, 1.0),
        R::atio(1, 2, -0.0, 0.5),
        R::atio(1, 2, 0.5, 0.5),
        R::atio(1, 4, 1.25, 0.6),
        R::atio(1, 4, 0.0, 0.6),
    ];

    (l_ratios, r_ratios)
}


pub fn simple_ratios() -> (Vec<R>, Vec<R>) {
    let l_ratios = vec![
//        R::atio(5, 4, -2.0, 1.0),
//        R::atio(5, 4, 0.0, 1.0),
        R::atio(1, 1, 0.0, 1.0),
        R::atio(1, 1, -0.4, 1.0),
    ];

    let r_ratios = vec![
//        R::atio(5, 4, -3.0, 1.0),
//        R::atio(5, 4, 5.0, 1.0),
        R::atio(1, 1, -0.5, 1.0),
        R::atio(1, 1, 0.25, 1.0),
    ];

    (l_ratios, r_ratios)
}

#[derive(Debug, Clone, PartialEq)]
pub struct R {
    pub decimal: f32,
    pub offset: f32,
    pub ratio: String,
    pub gain: f32,
    pub pan: Pan,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Pan {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StereoRatios {
    pub l_ratios: Vec<R>,
    pub r_ratios: Vec<R>,
}

impl R {
    pub fn atio(n: usize, d: usize, offset: f32, gain: f32, pan: Pan) -> R {
        if d == 0 {
            panic!(
                "Denominator of a Ratio cannot be 0. Failed at R.atio({}, {}, {}, {})",
                n, d, offset, gain
            );
        }
        R {
            decimal: n as f32 / d as f32,
            offset,
            ratio: [n.to_string(), d.to_string()].join("/"),
            gain,
            pan,
        }
    }
}

//pub fn complicated_ratios() -> StereoRatios {
//    let l_ratios = vec![
//        R::atio(23, 2, 0.0, 0.04, Pan::Left),
//        R::atio(23, 2, -0.1, 0.04, Pan::Left),
//        R::atio(19, 2, 0.0, 0.1), Pan::Left,
//        R::atio(19, 2, -0.2, 0.1, Pan::Left),
//        R::atio(15, 2, 18.0, 0.15, Pan::Left),
//        R::atio(15, 2, 0.0, 0.15),
//        R::atio(10, 2, -9.0, 0.15),
//        R::atio(7, 2, 1.0, 1.0),
//        R::atio(7, 2, 0.0, 1.0),
//        R::atio(3, 2, 3.0, 1.0),
//        R::atio(12, 4, 0.0, 1.0),
//        R::atio(15, 8, 0.0, 1.0),
//        R::atio(15, 8, 6.0, 1.0),
//        R::atio(1, 1, 0.0, 1.0),
//        R::atio(1, 1, -2.0, 1.0),
//        R::atio(1, 2, 0.0, 0.5),
//        R::atio(1, 2, 0.5, 0.5),
//        R::atio(1, 4, 1.0, 0.6),
//        R::atio(1, 4, 0.0, 0.6),
//    ];
//
//    let r_ratios = vec![
//        R::atio(21, 2, 0.0, 0.05),
//        R::atio(21, 2, 0.2, 0.05),
//        R::atio(17, 2, 0.0, 0.1),
//        R::atio(17, 2, 0.3, 0.1),
//        R::atio(13, 2, 0.0, 0.15),
//        R::atio(13, 2, -11.0, 0.15),
//        R::atio(11, 2, 5.0, 0.15),
//        R::atio(11, 2, 0.0, 0.15),
//        R::atio(12, 4, 0.0, 1.0),
//        R::atio(9, 4, 0.0, 1.0),
//        R::atio(9, 4, 6.0, 1.0),
//        R::atio(5, 4, 0.0, 1.0),
//        R::atio(7, 3, -3.0, 1.0),
//        R::atio(11, 8, 0.0, 1.0),
//        R::atio(1, 1, -3.0, 1.0),
//        R::atio(1, 2, -0.0, 0.5),
//        R::atio(1, 2, 0.5, 0.5),
//        R::atio(1, 4, 1.25, 0.6),
//        R::atio(1, 4, 0.0, 0.6),
//    ];
//
//    StereoRatios { l_ratios, r_ratios }
//}

pub fn simple_ratios() -> Vec<R> {
    vec![
        R::atio(1, 2, 0.0, 0.2, Pan::Right),
        R::atio(1, 2, 3.0, 0.2, Pan::Right),
        R::atio(1, 1, -1.0, 0.5, Pan::Left),
        R::atio(7, 4, 1.0, 0.8, Pan::Right),
        R::atio(7, 4, 0.0, 0.7, Pan::Right),
        R::atio(3, 2, 0.0, 0.4, Pan::Right),
        R::atio(3, 2, 4.0, 0.3, Pan::Right),
        R::atio(3, 2, 4.0, 0.3, Pan::Right),
        R::atio(12, 5, 11.0, 0.2, Pan::Right),
        R::atio(12, 5, 0.0, 0.2, Pan::Right),
        R::atio(15, 4, 6.0, 0.17, Pan::Right),
        R::atio(15, 4, 5.0, 0.15, Pan::Right),
        R::atio(23, 4, 6.0, 0.095, Pan::Right),
        R::atio(23, 4, 5.0, 0.095, Pan::Right),
        R::atio(27, 4, 9.0, 0.055, Pan::Right),
        R::atio(27, 4, 0.0, 0.055, Pan::Right),
        R::atio(31, 4, 0.25, 0.05, Pan::Right),
        R::atio(37, 4, 0.0, 0.05, Pan::Right),
        //
        R::atio(1, 2, 0.0, 0.8, Pan::Left),
        R::atio(1, 2, -3.0, 0.8, Pan::Left),
        R::atio(1, 1, -1.0, 0.5, Pan::Left),
        R::atio(5, 4, 1.0, 0.7, Pan::Left),
        R::atio(5, 4, 0.0, 0.8, Pan::Left),
        R::atio(11, 8, 1.0, 0.4, Pan::Left),
        R::atio(11, 8, -4.0, 0.4, Pan::Left),
        R::atio(13, 4, -13.0, 0.2, Pan::Left),
        R::atio(13, 4, 6.0, 0.2, Pan::Left),
        R::atio(17, 4, 3.0, 0.15, Pan::Left),
        R::atio(17, 4, 4.0, 0.15, Pan::Left),
        R::atio(21, 4, 11.0, 0.095, Pan::Left),
        R::atio(21, 4, 0.0, 0.095, Pan::Left),
        R::atio(25, 4, -7.0, 0.055, Pan::Left),
        R::atio(25, 4, 0.0, 0.055, Pan::Left),
        R::atio(30, 4, 0.25, 0.05, Pan::Left),
        R::atio(30, 4, 0.0, 0.05, Pan::Left),
    ]
}

pub fn init(num_l: usize, num_r: usize) -> Vec<R> {
    let mut result: Vec<R> = vec![];
    for _l in 0..num_l {
        result.push(R::atio(1, 1, 0.0, 0.0, Pan::Left));
    }

    for _r in 0..num_r {
        result.push(R::atio(1, 1, 0.0, 0.0, Pan::Right));
    }

    result
}

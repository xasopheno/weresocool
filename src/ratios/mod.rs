pub mod presets;

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


#[cfg(test)]
mod tests {
    use ratios::{Pan, R};

    #[test]
    fn test_ratios() {
        let ratio = R::atio(3, 2, 1.0, 1.0, Pan::Left);
        let expected = R {
            decimal: 3.0/ 2.0,
            offset: 1.0,
            gain: 1.0,
            pan: Pan::Left,
            ratio: String::from("3/2"),
        };

        assert_eq!(ratio, expected);
    }
}
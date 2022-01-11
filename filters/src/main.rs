// From https://gitlab.com/ruivieira/mentat - Apache 2.0
use rand::distributions::Normal as N;
use rand::prelude::*;
use std::f64::consts::PI;

macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !(($x - $y).abs() < $d || ($y - $x).abs() < $d) {
            panic!();
        }
    };
}

fn main() {}

fn mean(numbers: &Vec<f64>) -> f64 {
    numbers.iter().sum::<f64>() as f64 / numbers.len() as f64
}

pub struct Normal {
    mean: f64,
    std: f64,
}

impl Normal {
    pub fn new(mean: f64, std: f64) -> Normal {
        return Normal { mean, std };
    }

    pub fn sample(&mut self) -> f64 {
        let normal = N::new(self.mean, self.std);
        let v = normal.sample(&mut rand::thread_rng());
        return v;
    }

    pub fn logpdf(&mut self, x: f64) -> f64 {
        return -0.5 * (2.0 * PI).ln()
            - self.std.ln()
            - (x - self.mean).powi(2) / (2.0 * self.std * self.std);
    }

    pub fn pdf(&mut self, x: f64) -> f64 {
        return self.logpdf(x).exp();
    }
}

pub struct MonotonicCubicSpline {
    m_x: Vec<f64>,
    m_y: Vec<f64>,
    m_m: Vec<f64>,
}

impl MonotonicCubicSpline {
    pub fn new(x: &Vec<f64>, y: &Vec<f64>) -> MonotonicCubicSpline {
        assert!(
            x.len() == y.len() && x.len() >= 2 && y.len() >= 2,
            "Must have at least 2 control points."
        );

        let n = x.len();

        let mut secants = vec![0.0; n - 1];
        let mut slopes = vec![0.0; n];

        for i in 0..(n - 1) {
            let h = *x.get(i + 1).unwrap() - *x.get(i).unwrap();
            assert!(h > 0.0, "Control points must be monotonically increasing.");
            secants[i] = (*y.get(i + 1).unwrap() - *y.get(i).unwrap()) / h;
        }

        slopes[0] = secants[0];
        for i in 1..(n - 1) {
            slopes[i] = (secants[i - 1] + secants[i]) * 0.5;
        }
        slopes[n - 1] = secants[n - 2];

        for i in 0..(n - 1) {
            if secants[i] == 0.0 {
                slopes[i] = 0.0;
                slopes[i + 1] = 0.0;
            } else {
                let alpha = slopes[i] / secants[i];
                let beta = slopes[i + 1] / secants[i];
                let h = alpha.hypot(beta);
                if h > 9.0 {
                    let t = 3.0 / h;
                    slopes[i] = t * alpha * secants[i];
                    slopes[i + 1] = t * beta * secants[i];
                }
            }
        }

        let spline = MonotonicCubicSpline {
            m_x: x.clone(),
            m_y: y.clone(),
            m_m: slopes,
        };
        return spline;
    }

    pub fn hermite(point: f64, x: (f64, f64), y: (f64, f64), m: (f64, f64)) -> f64 {
        let h: f64 = x.1 - x.0;
        let t = (point - x.0) / h;
        return (y.0 * (1.0 + 2.0 * t) + h * m.0 * t) * (1.0 - t) * (1.0 - t)
            + (y.1 * (3.0 - 2.0 * t) + h * m.1 * (t - 1.0)) * t * t;
    }

    pub fn interpolate(&mut self, point: f64) -> f64 {
        let n = self.m_x.len();

        if point <= *self.m_x.get(0).unwrap() {
            return *self.m_y.get(0).unwrap();
        }
        if point >= *self.m_x.get(n - 1).unwrap() {
            return *self.m_y.get(n - 1).unwrap();
        }

        let mut i = 0;
        while point >= *self.m_x.get(i + 1).unwrap() {
            i += 1;
            if point == *self.m_x.get(i).unwrap() {
                return *self.m_y.get(i).unwrap();
            }
        }
        return MonotonicCubicSpline::hermite(
            point,
            (*self.m_x.get(i).unwrap(), *self.m_x.get(i + 1).unwrap()),
            (*self.m_y.get(i).unwrap(), *self.m_y.get(i + 1).unwrap()),
            (*self.m_m.get(i).unwrap(), *self.m_m.get(i + 1).unwrap()),
        );
    }

    fn partial(x: Vec<f64>, y: Vec<f64>) -> impl Fn(f64) -> f64 {
        move |p| {
            let mut spline = MonotonicCubicSpline::new(&x, &y);
            spline.interpolate(p)
        }
    }
}

#[cfg(test)]
mod test_normal {

    use super::*;

    #[test]
    fn sample_mean_std() {
        let n = 0..1000000;
        let mut normal = Normal::new(0.0, 1.0);
        let samples = n.map(|_i| normal.sample()).collect::<Vec<f64>>();
        let mu = mean(&samples);
        assert_delta!(0.0, mu, 1e-3);
    }

    #[test]
    fn logpdf() {
        let mut normal = Normal::new(0.0, 1.0);
        let _logpdf = normal.logpdf(0.0);
        print!("{:?}", _logpdf);
        assert_delta!(-0.9189385, _logpdf, 1e-5);
    }

    fn pdf() {
        let mut normal = Normal::new(0.0, 1.0);
        let _pdf = normal.pdf(0.0);
        print!("{:?}", _pdf);
        assert_delta!(0.3989423, _pdf, 1e-5);
    }

    #[test]
    fn interpolation() {
        let x = vec![0.0, 2.0, 3.0, 10.0];
        let y = vec![1.0, 4.0, 8.0, 10.5];

        let smooth = MonotonicCubicSpline::partial(x.clone(), y.clone());

        let mut x_interp = Vec::new();
        let mut y_interp = Vec::new();
        for i in 0..100 {
            let p = i as f64 / 10.0;
            x_interp.push(p);
            y_interp.push(smooth(p));
        }
        // let mut figure = Figure::new();
        // let points = scatter_plot::<f64, f64>(x, y, None);
        // let interpolation = line_plot::<f64, f64>(x_interp, y_interp, None);
        // figure.add_plot(points);
        // figure.add_plot(interpolation);
        // figure.save("./docs/figures/monotonic_cubic_spline.png", None);
    }
}

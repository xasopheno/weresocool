#[derive(Debug, Clone, Copy)]
pub struct Normalizer {
    pub x: MinMax,
    pub y: MinMax,
    pub z: MinMax,
}

#[derive(Debug, Clone, Copy)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}

impl Normalizer {
    pub const fn default() -> Self {
        Self {
            x: MinMax {
                min: -1.0,
                max: 1.0,
            },
            y: MinMax {
                min: 0.0,
                max: 2000.0,
            },
            z: MinMax { min: 0.0, max: 1.0 },
        }
    }
}

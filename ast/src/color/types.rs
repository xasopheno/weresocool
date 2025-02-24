use std::hash::{Hash, Hasher};
use bimap::BiHashMap;
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorValueMap {
    map: BiHashMap<u64, ColorValue>,
}

impl ColorValueMap {
    pub fn new() -> Self {
        Self {
            map: BiHashMap::new(),
        }
    }

    pub fn insert(&mut self, value: ColorValue) -> u64 {
        let hash = Self::calculate_hash(&value);
        if !self.map.contains_left(&hash) {
            self.map.insert(hash, value);
        }
        hash
    }

    pub fn get_by_hash(&self, hash: u64) -> Option<&ColorValue> {
        self.map.get_by_left(&hash)
    }

    pub fn get_hash(&self, value: &ColorValue) -> Option<u64> {
        self.map.get_by_right(value).copied()
    }

    fn calculate_hash(value: &ColorValue) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorValue {
    ColorGrad { colors: Vec<CssOrHex> },
    ColorSet { colors: Vec<CssOrHex> },
    Color(CssOrHex),
    RandColor(String),
}

impl Hash for ColorValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ColorValue::ColorGrad { colors } => {
                0.hash(state);
                colors.hash(state);
            }
            ColorValue::ColorSet { colors } => {
                1.hash(state);
                colors.hash(state);
            }
            ColorValue::Color(color) => {
                2.hash(state);
                color.hash(state);
            }
            ColorValue::RandColor(id) => {
                3.hash(state);
                id.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub enum CssOrHex {
    Css(String),
    Hex(String),
}

impl PartialEq for CssOrHex {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CssOrHex::Css(a), CssOrHex::Css(b)) => a.eq_ignore_ascii_case(b),
            (CssOrHex::Hex(a), CssOrHex::Hex(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        }
    }
}

impl Hash for CssOrHex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CssOrHex::Css(s) => {
                0.hash(state);
                s.to_lowercase().hash(state);
            }
            CssOrHex::Hex(s) => {
                1.hash(state);
                s.to_lowercase().hash(state);
            }
        }
    }
}

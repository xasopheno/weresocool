use std::str::FromStr;
use std::hash::{Hash, Hasher, DefaultHasher};
use colorgrad::Gradient;
use dyn_clone::DynClone;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::fmt::Debug;
use bimap::BiHashMap;
// use indexmap::IndexMap;

// pub type GenColorMap = IndexMap<String, Box<dyn GenColor>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorMap {
    pub map: BiHashMap<String, ColorValue>,
    next_id: u64,
}

impl ColorMap {
    pub fn new() -> Self {
        Self {
            map: BiHashMap::new(),
            next_id: 0,
        }
    }

    // ─────────────────────────────────────────────
    //  PUBLIC API (unchanged signatures)
    // ─────────────────────────────────────────────

    /// Return an id for the colour, reusing an existing one if present.
    pub fn insert(&mut self, value: ColorValue) -> u64 {
        // fast path: have we seen this colour before?
        if let Some(id_str) = self.map.get_by_right(&value) {
            return id_str.parse::<u64>().expect("ids stay numeric");
        }

        // new colour – assign next available id
        let id = self.next_id;
        self.next_id += 1;
        self.map.insert(id.to_string(), value);
        id
    }

    /// Associate an *explicit* name (“red”, “my_accent_colour”, …) with a colour.
    /// Overwrites silently if the name was already present.
    pub fn insert_by_name(&mut self, name: String, value: ColorValue) {
        self.map.insert(name, value);
    }

    /// Look up a colour by the string id you got from `insert()`.
    pub fn get_by_hash(&self, hash: String) -> Option<&ColorValue> {
        self.map.get_by_left(&hash)
    }

    /// Get the string id that `insert()` (or a previous call to this fn) produced
    /// for the given colour, if any.
    pub fn get_id_for_color(&self, color: &ColorValue) -> Option<String> {
        self.map.get_by_right(color).cloned()
    }
}

pub trait GenColor: DynClone + Debug + Send + Sync {
    fn gen(&self) -> Color;
    fn update(&mut self);
}
dyn_clone::clone_trait_object!(GenColor);

pub fn parse_color_set(colors: Vec<CssOrHex>) -> ColorValue {
    let color_strings: Vec<String> = colors
        .iter()
        .map(|color| match color {
            CssOrHex::Css(name) => name.clone(),
            CssOrHex::Hex(hex) => hex.clone(),
        })
        .collect();

    // Validate colors: Ensure they are all parseable
    for color in &color_strings {
        if csscolorparser::Color::from_str(color).is_err() {
            panic!("Invalid color: {}", color);
        }
    }

    let color_refs: Vec<&str> = color_strings.iter().map(|s| s.as_str()).collect();

    ColorValue::ColorSet {
        colors: vec_hex_to_vec_color(color_refs),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorValue {
    // ColorGrad { colors: Vec<CssOrHex> },
    // TODO: Should use the ColorSet directly
    ColorSet { colors: Vec<Color> },
    Color(CssOrHex),
    // RandColor(String),
}

impl Eq for Color {} 

impl Hash for ColorValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            // ColorValue::ColorGrad { colors } => {
                // 0.hash(state);
                // colors.hash(state);
            // }
            ColorValue::ColorSet { colors } => {
                1.hash(state);
                colors.hash(state);
            }
            ColorValue::Color(color) => {
                2.hash(state);
                color.hash(state);
            }
            // ColorValue::RandColor(id) => {
                // 3.hash(state);
                // id.hash(state);
            // }
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

impl CssOrHex {
    pub fn to_color(&self) -> [f32; 4] {
        let s = match self {
            CssOrHex::Css(s) => s,
            CssOrHex::Hex(s) => s,
        };

        let color = csscolorparser::Color::from_str(s).unwrap();

        color.to_array()
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

#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.to_bits().hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct RandColor;

impl GenColor for RandColor {
    fn gen(&self) -> Color {
        let mut rng = rand::thread_rng();
        let mut r = || rng.gen::<f32>() * 2.0 - 1.0;

        Color {
            r: r(),
            g: r(),
            b: r(),
            a: 1.0,
        }
    }
    fn update(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct RandColorSet {
    colors: Vec<Color>,
}

#[allow(dead_code)]
impl RandColorSet {
    pub fn init(n: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut r = || rng.gen::<f32>() * 2.0 - 1.0;

        RandColorSet {
            colors: (0..n)
                .map(|_| Color {
                    r: r(),
                    g: r(),
                    b: r(),
                    a: 1.0,
                })
                .collect(),
        }
    }
}

impl GenColor for RandColorSet {
    fn gen(&self) -> Color {
        self.colors
            .choose(&mut rand::thread_rng())
            .expect("Color choice failed")
            .clone()
    }
    fn update(&mut self) {}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorSet {
    pub colors: Vec<Color>,
}

impl<'a> ColorSet {
    pub fn init<T>(hex_strings: T) -> ColorSet
    where
        T: Into<Vec<&'a str>>,
    {
        ColorSet {
            colors: vec_hex_to_vec_color(hex_strings.into()),
        }
    }
}

pub fn vec_hex_to_vec_color(hex_strings: Vec<&str>) -> Vec<Color> {
    hex_strings.iter().map(|&s| parse_css_color(s)).collect()
}

impl GenColor for ColorSet {
    fn gen(&self) -> Color {
        self.colors
            .choose(&mut rand::thread_rng())
            .expect("Color choice failed")
            .clone()
    }
    fn update(&mut self) {}
}

/// Gradient color generator.
#[derive(Clone, Debug)]
pub struct GradientColor {
    colors: Vec<String>,
}

impl<'a> GradientColor {
    pub fn init<T>(hex_strings: T) -> GradientColor
    where
        T: Into<Vec<&'a str>>,
    {
        GradientColor {
            colors: hex_strings.into().iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl GenColor for GradientColor {
    fn gen(&self) -> Color {
        let gradient = colorgrad::GradientBuilder::new()
            .html_colors(self.colors.as_slice())
            .build::<colorgrad::LinearGradient>()
            .unwrap();

        let mut rng = thread_rng();
        let position: f32 = rng.gen_range(0.0, 1.0);
        let grad_color = gradient.at(position);

        Color {
            r: grad_color.r,
            g: grad_color.g,
            b: grad_color.b,
            a: grad_color.a,
        }
    }

    fn update(&mut self) {}
}

#[derive(Clone, Debug)]
pub struct ColorSets {
    current: usize,
    colorsets: Vec<Box<dyn GenColor>>,
}

#[allow(dead_code)]
impl ColorSets {
    pub fn new(colorsets: Vec<Box<dyn GenColor>>) -> Self {
        ColorSets {
            current: 0,
            colorsets,
        }
    }
}

impl GenColor for ColorSets {
    fn gen(&self) -> Color {
        self.colorsets[self.current].gen()
    }
    fn update(&mut self) {
        self.current = (self.current + 1) % self.colorsets.len();
    }
}

pub fn parse_css_color(s: &str) -> Color {
    match csscolorparser::Color::from_str(s) {
        Ok(parsed_color) => {
            let [r, g, b, a] = parsed_color.to_array();
            Color { r, g, b, a }
        }
        Err(_) => Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    }
}


#[macro_export]
macro_rules! color_gradient { ( $( $color:expr ),+ $(,)? ) => { GradientColor::init(vec![ $( $color ),+ ])
    };
}

#[macro_export]
macro_rules! color_set {
    ( $( $color:expr ),+ $(,)? ) => {
        ColorSet::init(vec![ $( $color ),+ ])
    };
}

#[macro_export]
macro_rules! rand_color {
    () => {
        RandColor
    };
}

#[macro_export]
macro_rules! color {
    (
        $(
            $name:ident : $value:expr
        ),* $(,)?
    ) => {{
        let mut map: ColorMap = IndexMap::new();
        $(
            map.insert(
                stringify!($name).to_string(),
                Box::new($value) as Box<dyn GenColor>
            );
        )*
        map
    }};
}




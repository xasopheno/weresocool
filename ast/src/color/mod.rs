use std::str::FromStr;
use std::hash::{Hash, Hasher};
use colorgrad::Gradient;
use dyn_clone::DynClone;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::fmt::Debug;
use bimap::BiHashMap;
use num_rational::Rational64;
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

    /// Associate an *explicit* name ("red", "my_accent_colour", …) with a colour.
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

    /// Update next_id to be at least the given value
    /// Used when merging ColorMaps to ensure unique IDs
    pub fn update_next_id(&mut self, next_id: u64) {
        if next_id > self.next_id {
            self.next_id = next_id;
        }
    }

    /// Get the current next_id value
    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

pub trait GenColor: DynClone + Debug + Send + Sync {
    fn generate_color(&self) -> Color;
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ColorValue {
    ColorSet { colors: Vec<Color> },
    Color(CssOrHex),
}

impl ColorValue {
    /// Extract a single Color from this ColorValue.
    /// For ColorSet, picks the first color (could be random).
    pub fn extract_color(&self) -> Color {
        match self {
            ColorValue::Color(css_or_hex) => {
                let s = match css_or_hex {
                    CssOrHex::Css(name) => name.as_str(),
                    CssOrHex::Hex(hex) => hex.as_str(),
                };
                parse_css_color(s)
            }
            ColorValue::ColorSet { colors } => {
                // Pick first color for deterministic results
                colors.first().cloned().unwrap_or(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub enum CssOrHex {
    Css(String),
    Hex(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NumberOrPoint {
    Number(f32),
    Point { value: f32, time: f32 },
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
        self.g.to_bits().hash(state);
        self.b.to_bits().hash(state);
        self.a.to_bits().hash(state);
    }
}

impl Eq for Color {}

#[derive(Clone, Debug)]
pub struct RandColor;

impl GenColor for RandColor {
    fn generate_color(&self) -> Color {
        let r = || rand::random::<f32>() * 2.0 - 1.0;

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
        let r = || rand::random::<f32>() * 2.0 - 1.0;

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
    fn generate_color(&self) -> Color {
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
    fn generate_color(&self) -> Color {
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
    fn generate_color(&self) -> Color {
        let gradient = colorgrad::GradientBuilder::new()
            .html_colors(self.colors.as_slice())
            .build::<colorgrad::LinearGradient>()
            .unwrap();

        let mut rng = thread_rng();
        let position: f32 = rng.gen_range(0.0..1.0);
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
    fn generate_color(&self) -> Color {
        self.colorsets[self.current].generate_color()
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

// ============================================================================
// Color Grading Functions
// ============================================================================

/// Apply color grading adjustments to a Color
/// Applies: Hue, Brightness, Gamma, Saturation, Vibrance
pub fn apply_color_grading(color: &Color, grading: &crate::operations::ColorGrading) -> Color {
    let mut c = color.clone();

    // Helper to convert Rational64 to f32
    let r64_to_f32 = |r: &Rational64| -> f32 {
        (*r.numer() as f64 / *r.denom() as f64) as f32
    };

    // 1. Brightness (additive offset)
    let brightness = r64_to_f32(&grading.brightness);
    c = apply_brightness(c, brightness);

    // 2. Gamma (power curve)
    let gamma = r64_to_f32(&grading.gamma);
    c = apply_gamma(c, gamma);

    // 3. Hue shift (in HSL space)
    let hue_shift = r64_to_f32(&grading.hue);
    c = apply_hue(c, hue_shift);

    // 4. Saturation (standard HSL saturation)
    let saturation = r64_to_f32(&grading.saturation);
    c = apply_saturation(c, saturation);

    // 5. Vibrance (smart saturation boost)
    let vibrance = r64_to_f32(&grading.vibrance);
    c = apply_vibrance(c, vibrance);

    c
}

fn apply_brightness(mut c: Color, brightness: f32) -> Color {
    // Simple additive offset to all RGB channels
    c.r = (c.r + brightness).clamp(0.0, 1.0);
    c.g = (c.g + brightness).clamp(0.0, 1.0);
    c.b = (c.b + brightness).clamp(0.0, 1.0);
    c
}

fn apply_gamma(mut c: Color, gamma: f32) -> Color {
    // Power curve adjustment
    // gamma < 1.0 = brighter, gamma > 1.0 = darker
    if gamma > 0.0 {
        c.r = c.r.powf(gamma).clamp(0.0, 1.0);
        c.g = c.g.powf(gamma).clamp(0.0, 1.0);
        c.b = c.b.powf(gamma).clamp(0.0, 1.0);
    }
    c
}

fn apply_hue(c: Color, hue_shift: f32) -> Color {
    // Convert to HSL, rotate hue, convert back
    let (h, s, l) = rgb_to_hsl(c.r, c.g, c.b);
    let new_h = (h + hue_shift).rem_euclid(1.0); // Wrap around 0..1
    let (r, g, b) = hsl_to_rgb(new_h, s, l);
    Color { r, g, b, a: c.a }
}

fn apply_vibrance(c: Color, vibrance: f32) -> Color {
    // Vibrance: boost saturation for less saturated colors
    let (h, s, l) = rgb_to_hsl(c.r, c.g, c.b);

    // Calculate boost amount: more boost for less saturated colors
    let boost = vibrance * (1.0 - s);
    let new_s = (s + boost).clamp(0.0, 1.0);

    let (r, g, b) = hsl_to_rgb(h, new_s, l);
    Color { r, g, b, a: c.a }
}

fn apply_saturation(c: Color, saturation: f32) -> Color {
    // Convert to HSL, adjust saturation, convert back
    let (h, s, l) = rgb_to_hsl(c.r, c.g, c.b);
    let new_s = (s * saturation).clamp(0.0, 1.0);
    let (r, g, b) = hsl_to_rgb(h, new_s, l);
    Color { r, g, b, a: c.a }
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, l); // achromatic
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if max == r {
        ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l); // achromatic
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };

    let p = 2.0 * l - q;

    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0/6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0/2.0 { return q; }
        if t < 2.0/3.0 { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
        p
    };

    let r = hue_to_rgb(p, q, h + 1.0/3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0/3.0);

    (r, g, b)
}

/// Convert a Color to a hex string
pub fn color_to_hex(color: &Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
        (color.a * 255.0).round() as u8,
    )
}

// ============================================================================
// Color Distribution - spatial arrangement of colors
// ============================================================================

/// Describes how colors are distributed spatially
#[derive(Clone, Debug)]
pub struct ColorDistribution {
    /// Direction vector for gradient distribution. None = pure random.
    pub gradient: Option<(f32, f32, f32)>,
    /// Mix amount: 0.0 = pure gradient, 1.0 = pure random
    pub mix: f32,
}

impl PartialEq for ColorDistribution {
    fn eq(&self, other: &Self) -> bool {
        self.gradient == other.gradient && self.mix == other.mix
    }
}

impl Eq for ColorDistribution {}

impl std::hash::Hash for ColorDistribution {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash gradient as bits
        match self.gradient {
            Some((x, y, z)) => {
                1u8.hash(state);
                x.to_bits().hash(state);
                y.to_bits().hash(state);
                z.to_bits().hash(state);
            }
            None => 0u8.hash(state),
        }
        self.mix.to_bits().hash(state);
    }
}

impl PartialOrd for ColorDistribution {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ColorDistribution {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare gradient first
        match (&self.gradient, &other.gradient) {
            (None, None) => {}
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some((ax, ay, az)), Some((bx, by, bz))) => {
                match ax.to_bits().cmp(&bx.to_bits()) {
                    std::cmp::Ordering::Equal => {}
                    ord => return ord,
                }
                match ay.to_bits().cmp(&by.to_bits()) {
                    std::cmp::Ordering::Equal => {}
                    ord => return ord,
                }
                match az.to_bits().cmp(&bz.to_bits()) {
                    std::cmp::Ordering::Equal => {}
                    ord => return ord,
                }
            }
        }
        self.mix.to_bits().cmp(&other.mix.to_bits())
    }
}

impl Default for ColorDistribution {
    fn default() -> Self {
        ColorDistribution {
            gradient: None,
            mix: 1.0, // default = pure random (current behavior)
        }
    }
}

impl ColorDistribution {
    /// Check if this is the default (pure random) distribution
    pub fn is_random(&self) -> bool {
        self.gradient.is_none()
    }
}

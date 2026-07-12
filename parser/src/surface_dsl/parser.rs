//! Hand-rolled tokenizer + recursive-descent parser for the surface DSL.
//!
//! Grammar:
//! ```text
//!   pipeline := source ("|" op)*
//!   source   := "Plane" "(" num "," num "," num ")"
//!             | "Sphere" "(" num "," num ")"
//!             | "Cylinder" "(" num "," num "," num ")"
//!   op       := "Wave2" "(" num "," num "," num ")"
//!             | "Noise" "(" num "," num ")"
//!             | "Bend" "(" axis "," num ")"
//!             | "Twist" "(" axis "," num ")"
//!             | "Smooth"
//!   axis     := "x" | "y" | "z" | "X" | "Y" | "Z"
//!   num      := INT ("/" INT)? | FLOAT      // a/b is a rational literal
//! ```
//!
//! Mirrors warp/parser.rs's tokenizer (same comment forms, same token set)
//! so a composer can read surface defs with the same eye that reads warps.

use super::ast::*;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(f32),
    Pipe,
    LParen, RParen,
    Comma,
    Minus,
    Slash,
}

/// Surface DSL parser errors go through `dsl_parse_error::DslParseError`
/// so they share the same pretty source-context display as warp, draw,
/// and weresocool. Improving error UX = editing `dsl_parse_error` once.
pub use crate::dsl_parse_error::DslParseError as ParseError;

/// Construct an unlocated surface parse error.
fn perr(msg: impl Into<String>) -> ParseError {
    ParseError::new("surface", msg)
}

fn tokenize(src: &str) -> Result<Vec<Tok>, ParseError> {
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_whitespace() { i += 1; continue; }
        // C-style comments
        if c == '/' && i + 1 < bytes.len() && (bytes[i+1] as char) == '/' {
            while i < bytes.len() && (bytes[i] as char) != '\n' { i += 1; }
            continue;
        }
        // socool-style comments
        if c == '-' && i + 1 < bytes.len() && (bytes[i+1] as char) == '-' {
            while i < bytes.len() && (bytes[i] as char) != '\n' { i += 1; }
            continue;
        }
        match c {
            '|' => { out.push(Tok::Pipe); i += 1; }
            '(' => { out.push(Tok::LParen); i += 1; }
            ')' => { out.push(Tok::RParen); i += 1; }
            ',' => { out.push(Tok::Comma); i += 1; }
            '-' => { out.push(Tok::Minus); i += 1; }
            '/' => { out.push(Tok::Slash); i += 1; }
            d if d.is_ascii_digit() || d == '.' => {
                // Number: optionally followed by `/INT` to form a rational
                // literal — keeps socool fractions like 1/4 readable as
                // surface parameters.
                let start = i;
                while i < bytes.len() && {
                    let b = bytes[i] as char;
                    b.is_ascii_digit() || b == '.'
                } { i += 1; }
                let lhs: f32 = src[start..i].parse().map_err(|_|
                    ParseError::at_byte("surface", start, "malformed number literal")
                        .with_source(src))?;
                if i < bytes.len() && (bytes[i] as char) == '/'
                    && i + 1 < bytes.len() && (bytes[i+1] as char).is_ascii_digit()
                {
                    i += 1; // consume slash
                    let s2 = i;
                    while i < bytes.len() && (bytes[i] as char).is_ascii_digit() { i += 1; }
                    let rhs: f32 = src[s2..i].parse().map_err(|_|
                        ParseError::at_byte("surface", s2,
                            "malformed denominator in rational literal")
                            .with_source(src))?;
                    out.push(Tok::Num(lhs / rhs));
                } else {
                    out.push(Tok::Num(lhs));
                }
            }
            a if a.is_ascii_alphabetic() || a == '_' => {
                let start = i;
                while i < bytes.len() && {
                    let b = bytes[i] as char;
                    b.is_ascii_alphanumeric() || b == '_'
                } { i += 1; }
                out.push(Tok::Ident(src[start..i].to_string()));
            }
            _ => return Err(ParseError::at_byte("surface", i,
                format!("unexpected character {:?}", c)).with_source(src)),
        }
    }
    Ok(out)
}

struct Parser { toks: Vec<Tok>, pos: usize }

impl Parser {
    fn peek(&self) -> Option<&Tok> { self.toks.get(self.pos) }
    fn bump(&mut self) -> Option<Tok> { let t = self.toks.get(self.pos).cloned(); self.pos += 1; t }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == Some(t) { self.pos += 1; true } else { false }
    }
    fn expect(&mut self, t: &Tok) -> Result<(), ParseError> {
        if self.eat(t) { Ok(()) }
        else { Err(perr(format!("expected {:?}, got {:?}", t, self.peek()))) }
    }

    /// Read a number (with optional unary minus). Allows negative literals
    /// for e.g. `Bend(y, -30)` to bend the other way.
    fn parse_num(&mut self) -> Result<f32, ParseError> {
        let sign = if self.eat(&Tok::Minus) { -1.0 } else { 1.0 };
        match self.bump() {
            Some(Tok::Num(n)) => Ok(sign * n),
            other => Err(perr(format!("expected number, got {:?}", other))),
        }
    }

    fn parse_axis(&mut self) -> Result<SurfaceAxis, ParseError> {
        match self.bump() {
            Some(Tok::Ident(s)) => match s.as_str() {
                "x" | "X" => Ok(SurfaceAxis::X),
                "y" | "Y" => Ok(SurfaceAxis::Y),
                "z" | "Z" => Ok(SurfaceAxis::Z),
                other => Err(perr(format!("unknown axis {:?} (expected x/y/z)", other))),
            },
            other => Err(perr(format!("expected axis, got {:?}", other))),
        }
    }

    fn parse_pipeline(&mut self) -> Result<SurfacePipeline, ParseError> {
        let source = self.parse_source()?;
        let mut ops = Vec::new();
        while self.eat(&Tok::Pipe) {
            ops.push(self.parse_op()?);
        }
        if self.peek().is_some() {
            return Err(perr(format!("trailing tokens after pipeline: {:?}", self.peek())));
        }
        Ok(SurfacePipeline { source, ops })
    }

    fn parse_source(&mut self) -> Result<SurfaceSource, ParseError> {
        let name = match self.bump() {
            Some(Tok::Ident(s)) => s,
            other => return Err(perr(format!("expected source name, got {:?}", other))),
        };
        match name.as_str() {
            "Plane" => {
                self.expect(&Tok::LParen)?;
                let width   = self.parse_num()?; self.expect(&Tok::Comma)?;
                let height  = self.parse_num()?; self.expect(&Tok::Comma)?;
                let subdivs = self.parse_num()? as u32;
                self.expect(&Tok::RParen)?;
                Ok(SurfaceSource::Plane { width, height, subdivs })
            }
            "Sphere" => {
                self.expect(&Tok::LParen)?;
                let radius  = self.parse_num()?; self.expect(&Tok::Comma)?;
                let subdivs = self.parse_num()? as u32;
                self.expect(&Tok::RParen)?;
                Ok(SurfaceSource::Sphere { radius, subdivs })
            }
            "Cylinder" => {
                self.expect(&Tok::LParen)?;
                let radius  = self.parse_num()?; self.expect(&Tok::Comma)?;
                let height  = self.parse_num()?; self.expect(&Tok::Comma)?;
                let subdivs = self.parse_num()? as u32;
                self.expect(&Tok::RParen)?;
                Ok(SurfaceSource::Cylinder { radius, height, subdivs })
            }
            other => Err(perr(format!(
                "unknown source {:?} (expected Plane | Sphere | Cylinder)", other))),
        }
    }

    fn parse_op(&mut self) -> Result<SurfaceOp, ParseError> {
        let name = match self.bump() {
            Some(Tok::Ident(s)) => s,
            other => return Err(perr(format!("expected op name, got {:?}", other))),
        };
        match name.as_str() {
            "Wave2" => {
                self.expect(&Tok::LParen)?;
                let fx  = self.parse_num()?; self.expect(&Tok::Comma)?;
                let fy  = self.parse_num()?; self.expect(&Tok::Comma)?;
                let amp = self.parse_num()?;
                self.expect(&Tok::RParen)?;
                Ok(SurfaceOp::Wave2 { fx, fy, amp })
            }
            "Noise" => {
                self.expect(&Tok::LParen)?;
                let scale = self.parse_num()?; self.expect(&Tok::Comma)?;
                let amp   = self.parse_num()?;
                self.expect(&Tok::RParen)?;
                Ok(SurfaceOp::Noise { scale, amp })
            }
            "Bend" => {
                self.expect(&Tok::LParen)?;
                let axis = self.parse_axis()?; self.expect(&Tok::Comma)?;
                let angle_deg = self.parse_num()?;
                self.expect(&Tok::RParen)?;
                Ok(SurfaceOp::Bend { axis, angle_deg })
            }
            "Twist" => {
                self.expect(&Tok::LParen)?;
                let axis = self.parse_axis()?; self.expect(&Tok::Comma)?;
                let strength = self.parse_num()?;
                self.expect(&Tok::RParen)?;
                Ok(SurfaceOp::Twist { axis, strength })
            }
            "Smooth" => Ok(SurfaceOp::Smooth),
            other => Err(perr(format!(
                "unknown op {:?} (expected Wave2 | Noise | Bend | Twist | Smooth)", other))),
        }
    }
}

/// Parse a surface-pipeline source string (the BODY between `{` and `}`).
/// The caller is responsible for extracting that body from a top-level
/// `surface NAME = { … }` declaration; this function takes just the inner
/// pipeline text.
pub fn parse_surface_pipeline(src: &str) -> Result<SurfacePipeline, ParseError> {
    let toks = tokenize(src).map_err(|e| e.with_source(src))?;
    let mut p = Parser { toks, pos: 0 };
    p.parse_pipeline().map_err(|e| e.with_source(src))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plane_only() {
        let p = parse_surface_pipeline("Plane(10, 10, 128)").unwrap();
        assert_eq!(p.source, SurfaceSource::Plane { width: 10.0, height: 10.0, subdivs: 128 });
        assert!(p.ops.is_empty());
    }

    #[test]
    fn parse_with_wave_and_smooth() {
        let p = parse_surface_pipeline("Plane(10, 10, 128) | Wave2(0.7, 0.5, 0.45) | Smooth").unwrap();
        assert_eq!(p.ops.len(), 2);
        assert!(matches!(p.ops[0], SurfaceOp::Wave2 { .. }));
        assert!(matches!(p.ops[1], SurfaceOp::Smooth));
    }

    #[test]
    fn parse_sphere_cylinder() {
        let p = parse_surface_pipeline("Sphere(3, 64)").unwrap();
        assert!(matches!(p.source, SurfaceSource::Sphere { radius: r, .. } if (r - 3.0).abs() < 1e-5));
        let q = parse_surface_pipeline("Cylinder(2, 5, 64) | Smooth").unwrap();
        assert!(matches!(q.source, SurfaceSource::Cylinder { .. }));
    }

    #[test]
    fn parse_bend_twist_with_axis() {
        let p = parse_surface_pipeline("Plane(8, 8, 64) | Bend(y, 30) | Twist(x, 0.5)").unwrap();
        assert!(matches!(p.ops[0], SurfaceOp::Bend { axis: SurfaceAxis::Y, .. }));
        assert!(matches!(p.ops[1], SurfaceOp::Twist { axis: SurfaceAxis::X, .. }));
    }

    #[test]
    fn parse_negative_angle() {
        let p = parse_surface_pipeline("Plane(8, 8, 64) | Bend(y, -45)").unwrap();
        if let SurfaceOp::Bend { angle_deg, .. } = p.ops[0] {
            assert!((angle_deg + 45.0).abs() < 1e-5);
        } else { panic!("expected Bend") }
    }

    #[test]
    fn comments_and_whitespace() {
        let p = parse_surface_pipeline(r#"
            -- a ripple
            Plane(10, 10, 128)
            | Wave2(0.7, 0.5, 0.45)   -- low-freq
            | Wave2(1.3, 0.8, 0.22)   // high-freq
            | Smooth
        "#).unwrap();
        assert_eq!(p.ops.len(), 3);
    }
}

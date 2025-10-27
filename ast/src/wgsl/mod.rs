use std::hash::{DefaultHasher, Hash, Hasher};
use std::collections::HashMap;
use naga::front::wgsl::Frontend;

pub const MAX_STEPS: u32 = 4; // compile-time bound for recipe size

#[derive(Clone, Debug, PartialEq)]
pub struct WgslMap {
    pub map: HashMap<u8, String>,
    next_id: u8,
}

impl WgslMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_id: 1,
        }
    }

    /// Insert WGSL code and return its sequential u8 ID
    pub fn insert(&mut self, code: String) -> u8 {
        let id = self.next_id;
        self.map.insert(id, code);
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Retrieve WGSL code by u8 ID
    pub fn get(&self, id: &u8) -> Option<&String> {
        self.map.get(id)
    }

    /// Update next_id to be at least the given value
    /// Used when merging WgslMaps to ensure unique IDs
    pub fn update_next_id(&mut self, next_id: u8) {
        if next_id > self.next_id {
            self.next_id = next_id;
        }
    }

    /// Get the current next_id value
    pub fn next_id(&self) -> u8 {
        self.next_id
    }
}

// Helper function to prepare WGSL code for naga validation
pub fn prepare_for_naga(src: &str) -> String {
    let mut out = String::new();
    
    // Define helper functions at global scope
    out.push_str("
// Helper functions
fn cos(v: f32) -> f32 { return 1.0; }
fn sin(v: f32) -> f32 { return 0.0; }
fn sqrt(v: f32) -> f32 { return 1.0; }
fn pow(v: f32, p: f32) -> f32 { return 1.0; }
fn abs(v: f32) -> f32 { return 1.0; }
fn min(a: f32, b: f32) -> f32 { return a; }
fn max(a: f32, b: f32) -> f32 { return a; }
fn floor(v: f32) -> f32 { return 1.0; }
fn ceil(v: f32) -> f32 { return 1.0; }
fn fract(v: f32) -> f32 { return 0.0; }

fn dummy_function() {
    // Variables that are modifiable
    var x: f32 = 0.0;
    var y: f32 = 0.0;
    var z: f32 = 0.0;
    var r: f32 = 0.01;
    var life: f32 = 1.0;
    var scale: f32 = 1.0;
    var time: f32 = 0.0;
");

    // Add the user's code
    out.push_str(src);
    
    // Close the function
    out.push_str("\n}\n");
    
    out
}

// Validate WGSL code
pub fn validate_wgsl(src: &str) -> Result<(), String> {
    let patched = prepare_for_naga(src);
    let mut parser = Frontend::new();
    match parser.parse(&patched) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.emit_to_string(&patched)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_wgsl() {
        let code = r#"
            // Spiral effect with time-based variations
            var new_x = x;
            var new_y = y;
            var new_z = z;

            // Apply spiral effect
            let angle = time * 0.1;
            new_x = x * cos(angle) - y * sin(angle);
            new_y = x * sin(angle) + y * cos(angle);

            // Add z oscillation
            new_z = z + sin(time * 0.5) * 0.2;

            // Apply modifications
            x = new_x;
            y = new_y;
            z = new_z;

            // Adjust rotation speed based on time
            r = 0.01 + sin(time * 0.05) * 0.005;

            // Life slowly decays
            life = life * 0.9998;

            // Scale pulses over time
            let pulse = sin(time * 0.1) * 0.5 + 0.5;
            scale = scale * (1.0 + pulse * 0.05) * 10.0;
        "#;

        assert!(validate_wgsl(code).is_ok());
    }

    #[test]
    fn test_invalid_wgsl() {
        let code = r#"
            // This contains a syntax error:
            let x = 1.0  // Missing semicolon
            y = 2.0;
        "#;

        assert!(validate_wgsl(code).is_err());
    }
    
    #[test]
    fn test_wgsl_map() {
        let mut map = WgslMap::new();
        let code = "x = x * 2.0;";
        let id = map.insert(code.to_string());
        assert_eq!(id, 1, "First ID should be 1");
        assert_eq!(map.get(&id), Some(&code.to_string()));
        let code2 = "y = y + 1.0;";
        let id2 = map.insert(code2.to_string());
        assert_eq!(id2, 2, "Second ID should be 2");
        assert_eq!(map.get(&id2), Some(&code2.to_string()));
    }
} 

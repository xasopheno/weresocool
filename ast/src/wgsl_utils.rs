use naga::front::wgsl::Frontend;
use std::fs;

pub const MAX_STEPS: u32 = 4; // compile-time bound for recipe size

// Helper function to prepare WGSL code for naga validation
pub fn prepare_for_naga(src: &str) -> String {
    let mut out = String::new();
    
    // Add necessary variable declarations that would be available in the shader
    out.push_str("
// Variables that would be available in the actual shader
var x: f32 = 0.0;
var y: f32 = 0.0;
var z: f32 = 0.0;
var r: f32 = 0.0;
var life: f32 = 1.0;
var scale: f32 = 1.0;
var time: f32 = 0.0;

fn dummy_function() {
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
    fn test_complex_wgsl() {
        let code = r#"
            // Complex particle transformation
            // Calculate distance from origin
            let dist = sqrt(x*x + y*y + z*z);
            
            // Apply rotation around Y axis
            let theta = time * 0.2;
            let new_x = x * cos(theta) - z * sin(theta);
            let new_z = x * sin(theta) + z * cos(theta);
            
            // Apply a wave effect based on distance
            let wave = sin(dist * 3.0 + time) * 0.1;
            
            // Scale based on life and time
            let scalar = life * (0.8 + sin(time * 0.3) * 0.2);
            
            // Apply transformations
            x = new_x * scalar + wave;
            y = y * scalar + wave;
            z = new_z * scalar;
            
            // Adjust scale based on distance
            scale = 1.0 + dist * 0.05;
            
            // Slowly decrease life
            life = life * 0.999;
            
            // Adjust rotation speed
            r = 0.01 + dist * 0.002;
        "#;

        assert!(validate_wgsl(code).is_ok());
    }
} 

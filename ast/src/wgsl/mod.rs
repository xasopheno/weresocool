use std::collections::HashMap;
use naga::front::wgsl::Frontend;
use colored::*;

pub const MAX_STEPS: u32 = 4; // compile-time bound for recipe size

/// Error from WGSL validation with position mapped to original source
#[derive(Clone, Debug)]
pub struct WgslError {
    pub message: String,
    pub line: usize,      // Line in original source (1-based)
    pub column: usize,    // Column in original source (1-based)
    pub context: String,  // The problematic line of code
}

impl WgslError {
    /// Display the error with colored output matching the main parser style
    pub fn display_colored(&self, original_source: &str) {
        let start_offset = 125;
        let end_offset = 50;

        // Find the byte offset in original_source for the error line
        // Count newlines until we reach the target line
        let mut current_line = 0;
        let mut line_start = 0;
        for (i, c) in original_source.char_indices() {
            if c == '\n' {
                current_line += 1;
                if current_line == self.line {
                    // The line content starts after this newline
                    line_start = i + 1;
                    break;
                }
            }
        }
        let error_pos = line_start + self.column.saturating_sub(1);

        // Calculate display window
        let feed_start = error_pos.saturating_sub(start_offset);
        let mut feed_end = (error_pos + end_offset).min(original_source.len());
        if feed_end - feed_start > 300 {
            feed_end = feed_start + 300;
        }

        // Show context with colors: light blue before error, red from error
        // Using cyan/bright_blue to distinguish WGSL errors from regular parse errors
        println!(
            "{}{}",
            &original_source[feed_start..error_pos].cyan(),
            &original_source[error_pos..feed_end].red(),
        );

        println!(
            "
            {}
            WGSL errors at line {}
            {}
            ",
            "working".cyan().underline(),
            self.line.to_string().red().bold(),
            "broken".red().underline(),
        );
    }

    pub fn display(&self) -> String {
        format!(
            "WGSL error at line {}, column {}:\n  {}\n  {}^ {}",
            self.line,
            self.column,
            self.context.trim_end(),
            " ".repeat(self.column.saturating_sub(1)),
            self.message
        )
    }
}

/// Entry storing both compiled WGSL and original source (for formatting)
#[derive(Clone, Debug, PartialEq)]
pub struct WgslEntry {
    /// Compiled WGSL code (used at runtime)
    pub compiled: String,
    /// Original source code (may include DSL syntax like Ym 2/3)
    pub original: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WgslMap {
    pub map: HashMap<u8, WgslEntry>,
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
    /// Both the compiled and original source are stored
    pub fn insert(&mut self, compiled: String, original: String) -> u8 {
        let id = self.next_id;
        self.map.insert(id, WgslEntry { compiled, original });
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Insert with same source for both compiled and original (legacy interface)
    pub fn insert_raw(&mut self, code: String) -> u8 {
        self.insert(code.clone(), code)
    }

    /// Retrieve compiled WGSL code by u8 ID
    pub fn get(&self, id: &u8) -> Option<&String> {
        self.map.get(id).map(|e| &e.compiled)
    }

    /// Retrieve original source code by u8 ID (for formatting)
    pub fn get_original(&self, id: &u8) -> Option<&String> {
        self.map.get(id).map(|e| &e.original)
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
// Returns (patched_code, preamble_line_count)
pub fn prepare_for_naga(src: &str) -> (String, usize) {
    let mut out = String::new();

    // Define helper functions at global scope
    let preamble = "
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
    var velocity: f32 = 1.0;
    var red: f32 = 1.0;
    var green: f32 = 1.0;
    var blue: f32 = 1.0;
    var alpha: f32 = 1.0;
";

    // Count preamble lines (lines before user code)
    let preamble_lines = preamble.chars().filter(|&c| c == '\n').count();

    out.push_str(preamble);
    // Add the user's code
    out.push_str(src);

    // Close the function
    out.push_str("\n}\n");

    (out, preamble_lines)
}

// Validate WGSL code (legacy interface)
pub fn validate_wgsl(src: &str) -> Result<(), String> {
    let (patched, _) = prepare_for_naga(src);
    let mut parser = Frontend::new();
    match parser.parse(&patched) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.emit_to_string(&patched)),
    }
}

/// Validate WGSL code and return errors with positions mapped to original source
///
/// Note: DSL syntax should already be compiled to WGSL before calling this function.
/// DSL compilation is done in the parser crate during process_wgsl_blocks.
///
/// # Arguments
/// * `src` - The WGSL code to validate (already compiled from DSL if applicable)
/// * `original_line` - The 1-based line number where the WGSL block starts in the original source
/// * `original_source` - The original full source (for extracting context)
pub fn validate_wgsl_with_position(
    src: &str,
    original_line: usize,
    original_source: &str,
) -> Result<(), WgslError> {
    let (patched, preamble_lines) = prepare_for_naga(src);
    let mut parser = Frontend::new();

    match parser.parse(&patched) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Get structured location info from naga
            let (error_line, error_column) = if let Some(loc) = e.location(&patched) {
                (loc.line_number as usize, loc.line_position as usize)
            } else {
                (1, 1)
            };

            // Map line number back to original source:
            // error_line is in the patched source (with preamble)
            // original_line is the line number of "wgsl {" in the source
            // User line 1 is at original_line + 1, user line 2 at original_line + 2, etc.
            // So: mapped_line = original_line + (error_line - preamble_lines)
            let mapped_line = if error_line > preamble_lines {
                original_line + (error_line - preamble_lines)
            } else {
                // Error is in preamble (shouldn't happen normally)
                original_line
            };

            // Extract the context line from original source
            // Note: original_source (composition) starts with \n, so line N is at index N in .lines()
            let context = original_source
                .lines()
                .nth(mapped_line)
                .unwrap_or("")
                .to_string();

            Err(WgslError {
                message: e.message().to_string(),
                line: mapped_line,
                column: error_column,
                context,
            })
        }
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
        let compiled = "x = x * 2.0;";
        let original = "Xm 2";
        let id = map.insert(compiled.to_string(), original.to_string());
        assert_eq!(id, 1, "First ID should be 1");
        assert_eq!(map.get(&id), Some(&compiled.to_string()));
        assert_eq!(map.get_original(&id), Some(&original.to_string()));
        let code2 = "y = y + 1.0;";
        let id2 = map.insert_raw(code2.to_string());
        assert_eq!(id2, 2, "Second ID should be 2");
        assert_eq!(map.get(&id2), Some(&code2.to_string()));
        // When using insert_raw, original == compiled
        assert_eq!(map.get_original(&id2), Some(&code2.to_string()));
    }
} 

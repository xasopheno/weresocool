use crate::parser::process_wgsl_blocks;
use weresocool_ast::Defs;

#[test]
fn test_extract_nested_wgsl_blocks() {
    // Test case with WGSL blocks nested inside SoCool syntax
    let input = r#"
main {
  thing 
  | wgsl {
      // Variables modified directly in transform function body
      x = x * 2.0;
      y = y + sin(time);
    }
  | wgsl {
      // More variable modifications
      z = z - 0.5;
      life = life * 0.99;
    },
  thing3 | wgsl {
      // Scale and rotation modifications
      scale = scale * 1.1;
      r = r + 0.01;
    }
}
"#;

    let mut defs = Defs::default();
    // Skip validation for tests
    let (result, _source_map) = process_wgsl_blocks(input, &mut defs, true, true).unwrap();

    // Check that the WGSL blocks were extracted and replaced with tokens
    assert!(result.contains("@WGSL@"));
    assert!(!result.contains("x = x * 2.0"));
    assert!(!result.contains("z = z - 0.5"));
    assert!(!result.contains("scale = scale * 1.1"));
    
    // Check that all 3 WGSL blocks were extracted
    assert_eq!(defs.wgsl.map.len(), 3);
    
    // Verify the extracted code is correct
    let wgsl_codes: Vec<&String> = defs.wgsl.map.values().map(|e| &e.compiled).collect();
    assert!(wgsl_codes.iter().any(|code| code.contains("x = x * 2.0")));
    assert!(wgsl_codes.iter().any(|code| code.contains("z = z - 0.5")));
    assert!(wgsl_codes.iter().any(|code| code.contains("scale = scale * 1.1")));
}

#[test]
fn test_wgsl_with_nested_braces() {
    // Test case with WGSL that contains its own braces
    let input = r#"
main {
  thing | wgsl {
      // Control flow with nested braces
      if (time > 10.0) {
          x = x * 2.0;
      } else {
          x = x * 0.5;
      }
    }
}
"#;

    let mut defs = Defs::default();
    // Skip validation for tests
    let (result, _source_map) = process_wgsl_blocks(input, &mut defs, true, true).unwrap();

    // Check that the WGSL block was extracted
    assert!(result.contains("@WGSL@"));
    assert!(!result.contains("if (time > 10.0)"));
    
    // Check that exactly 1 WGSL block was extracted
    assert_eq!(defs.wgsl.map.len(), 1);
    
    // Verify the extracted code includes the nested braces
    let wgsl_code = defs.wgsl.map.values().next().unwrap();
    assert!(wgsl_code.compiled.contains("if (time > 10.0) {"));
    assert!(wgsl_code.compiled.contains("} else {"));
}

#[test]
fn test_multiple_wgsl_blocks_same_line() {
    // Test case with multiple WGSL blocks on the same line
    let input = r#"
main {
  thing | wgsl { x = 1.0; } | wgsl { y = 2.0; },
  other | wgsl { z = 3.0; }
}
"#;

    let mut defs = Defs::default();
    // Skip validation for tests
    let (_result, _source_map) = process_wgsl_blocks(input, &mut defs, true, true).unwrap();

    // Check that all WGSL blocks were extracted
    assert_eq!(defs.wgsl.map.len(), 3);
    
    // Verify the extracted code is correct
    let wgsl_codes: Vec<&String> = defs.wgsl.map.values().map(|e| &e.compiled).collect();
    assert!(wgsl_codes.iter().any(|code| code.contains("x = 1.0")));
    assert!(wgsl_codes.iter().any(|code| code.contains("y = 2.0")));
    assert!(wgsl_codes.iter().any(|code| code.contains("z = 3.0")));
} 
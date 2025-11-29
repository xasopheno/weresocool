use std::collections::HashMap;
use naga::front::wgsl::Frontend;
use colored::*;
use num_rational::Rational64;

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

// ============================================================================
// VisualOp - AST for WGSL DSL
// ============================================================================

/// Visual operation AST - parsed from DSL, generates WGSL code
#[derive(Clone, Debug, PartialEq)]
pub enum VisualOp {
    /// Simple operation with modifiers
    Simple {
        x_mul: Option<Rational64>,
        x_add: Option<Rational64>,
        y_mul: Option<Rational64>,
        y_add: Option<Rational64>,
        z_mul: Option<Rational64>,
        z_add: Option<Rational64>,
        direction: Option<(Rational64, Rational64, Rational64)>,
        scale_mul: Option<Rational64>,
        scale_add: Option<Rational64>,
        velocity_mul: Option<Rational64>,
        velocity_add: Option<Rational64>,
        /// Bend: (bend_vector_x, bend_vector_y, bend_vector_z, strength)
        /// Creates a curved path that bulges toward bend_vector while maintaining direction
        bend: Option<(Rational64, Rational64, Rational64, Rational64)>,
        /// Alpha set: sets alpha directly (Alpha 0 = invisible, Alpha 1 = visible)
        alpha_set: Option<Rational64>,
        /// Alpha multiply: multiplies alpha (Am 0.5 = fade to 50%)
        alpha_mul: Option<Rational64>,
        length: Rational64, // duration in seconds (Lm modifier)
    },
    /// Sequence of operations (time-divided)
    Seq {
        items: Vec<VisualOp>,
    },
    /// Compose multiple ops (apply in order)
    Compose {
        operations: Vec<VisualOp>,
    },
}

impl Default for VisualOp {
    fn default() -> Self {
        VisualOp::Simple {
            x_mul: None,
            x_add: None,
            y_mul: None,
            y_add: None,
            z_mul: None,
            z_add: None,
            direction: None,
            scale_mul: None,
            scale_add: None,
            velocity_mul: None,
            velocity_add: None,
            bend: None,
            alpha_set: None,
            alpha_mul: None,
            length: Rational64::new(1, 1),
        }
    }
}

impl VisualOp {
    /// Get the length (duration) of this operation in seconds
    pub fn length(&self) -> Rational64 {
        match self {
            VisualOp::Simple { length, .. } => *length,
            VisualOp::Seq { items } => items.iter().map(|op| op.length()).sum(),
            VisualOp::Compose { operations } => {
                // Compose takes the max length of all operations
                operations
                    .iter()
                    .map(|op| op.length())
                    .max()
                    .unwrap_or_else(|| Rational64::new(1, 1))
            }
        }
    }

    /// Get the last direction from this operation (for Seq continuation)
    pub fn last_direction(&self) -> Option<(Rational64, Rational64, Rational64)> {
        match self {
            VisualOp::Simple { direction, .. } => *direction,
            VisualOp::Seq { items } => items.last().and_then(|op| op.last_direction()),
            VisualOp::Compose { operations } => {
                // Find the last operation that has a direction
                operations.iter().rev().find_map(|op| op.last_direction())
            }
        }
    }

    /// Set the length for a Simple variant
    pub fn with_length(self, new_length: Rational64) -> Self {
        match self {
            VisualOp::Simple {
                x_mul,
                x_add,
                y_mul,
                y_add,
                z_mul,
                z_add,
                direction,
                scale_mul,
                scale_add,
                velocity_mul,
                velocity_add,
                bend,
                alpha_set,
                alpha_mul,
                ..
            } => VisualOp::Simple {
                x_mul,
                x_add,
                y_mul,
                y_add,
                z_mul,
                z_add,
                direction,
                scale_mul,
                scale_add,
                velocity_mul,
                velocity_add,
                bend,
                alpha_set,
                alpha_mul,
                length: new_length,
            },
            // For Seq/Compose, we could scale all children, but for now just return unchanged
            other => other,
        }
    }

    /// Compose: self | other
    /// Applies `other` to `self`
    pub fn compose(self, other: VisualOp) -> VisualOp {
        match (self, other) {
            // Simple | Simple → merge fields
            (
                VisualOp::Simple {
                    x_mul: x_mul1,
                    x_add: x_add1,
                    y_mul: y_mul1,
                    y_add: y_add1,
                    z_mul: z_mul1,
                    z_add: z_add1,
                    direction: dir1,
                    scale_mul: scale_mul1,
                    scale_add: scale_add1,
                    velocity_mul: vel_mul1,
                    velocity_add: vel_add1,
                    bend: bend1,
                    alpha_set: alpha_set1,
                    alpha_mul: alpha_mul1,
                    length: len1,
                },
                VisualOp::Simple {
                    x_mul: x_mul2,
                    x_add: x_add2,
                    y_mul: y_mul2,
                    y_add: y_add2,
                    z_mul: z_mul2,
                    z_add: z_add2,
                    direction: dir2,
                    scale_mul: scale_mul2,
                    scale_add: scale_add2,
                    velocity_mul: vel_mul2,
                    velocity_add: vel_add2,
                    bend: bend2,
                    alpha_set: alpha_set2,
                    alpha_mul: alpha_mul2,
                    length: len2,
                },
            ) => VisualOp::Simple {
                x_mul: compose_mul(x_mul1, x_mul2),
                x_add: compose_add(x_add1, x_add2),
                y_mul: compose_mul(y_mul1, y_mul2),
                y_add: compose_add(y_add1, y_add2),
                z_mul: compose_mul(z_mul1, z_mul2),
                z_add: compose_add(z_add1, z_add2),
                direction: dir2.or(dir1), // Later wins
                scale_mul: compose_mul(scale_mul1, scale_mul2),
                scale_add: compose_add(scale_add1, scale_add2),
                velocity_mul: compose_mul(vel_mul1, vel_mul2),
                velocity_add: compose_add(vel_add1, vel_add2),
                bend: bend2.or(bend1), // Later wins
                alpha_set: alpha_set2.or(alpha_set1), // Later wins
                alpha_mul: compose_mul(alpha_mul1, alpha_mul2),
                length: len1 * len2, // Multiply lengths
            },

            // Seq | Simple → apply Simple's length modifier to the Seq
            (VisualOp::Seq { items }, VisualOp::Simple { length, .. }) => {
                // Scale all items by the length modifier
                let scaled_items: Vec<VisualOp> = items
                    .into_iter()
                    .map(|op| {
                        let new_len = op.length() * length;
                        op.with_length(new_len)
                    })
                    .collect();
                VisualOp::Seq { items: scaled_items }
            }

            // Seq | Seq → Compose
            (seq1 @ VisualOp::Seq { .. }, seq2 @ VisualOp::Seq { .. }) => VisualOp::Compose {
                operations: vec![seq1, seq2],
            },

            // Simple | Seq → apply Simple's length modifier to Seq, keep other Simple fields
            (
                VisualOp::Simple {
                    x_mul,
                    x_add,
                    y_mul,
                    y_add,
                    z_mul,
                    z_add,
                    direction,
                    scale_mul,
                    scale_add,
                    velocity_mul,
                    velocity_add,
                    bend,
                    alpha_set,
                    alpha_mul,
                    length,
                },
                VisualOp::Seq { items },
            ) => {
                // Scale all Seq items by the Simple's length modifier
                let scaled_items: Vec<VisualOp> = items
                    .into_iter()
                    .map(|op| {
                        let new_len = op.length() * length;
                        op.with_length(new_len)
                    })
                    .collect();
                let scaled_seq = VisualOp::Seq { items: scaled_items };

                // Create a Simple with the other fields (length already applied to Seq)
                let simple_remainder = VisualOp::Simple {
                    x_mul,
                    x_add,
                    y_mul,
                    y_add,
                    z_mul,
                    z_add,
                    direction,
                    scale_mul,
                    scale_add,
                    velocity_mul,
                    velocity_add,
                    bend,
                    alpha_set,
                    alpha_mul,
                    length: Rational64::new(1, 1), // Length already applied
                };

                // If Simple has meaningful fields, wrap both in Compose
                // Otherwise just return the scaled Seq
                let has_fields = x_mul.is_some()
                    || x_add.is_some()
                    || y_mul.is_some()
                    || y_add.is_some()
                    || z_mul.is_some()
                    || z_add.is_some()
                    || direction.is_some()
                    || scale_mul.is_some()
                    || scale_add.is_some()
                    || velocity_mul.is_some()
                    || velocity_add.is_some()
                    || bend.is_some()
                    || alpha_set.is_some()
                    || alpha_mul.is_some();

                if has_fields {
                    VisualOp::Compose {
                        operations: vec![simple_remainder, scaled_seq],
                    }
                } else {
                    scaled_seq
                }
            },

            // Compose | anything → add to existing compose
            (VisualOp::Compose { mut operations }, other) => {
                operations.push(other);
                VisualOp::Compose { operations }
            }

            // anything | Compose → create new compose
            (other, VisualOp::Compose { operations }) => {
                let mut new_ops = vec![other];
                new_ops.extend(operations);
                VisualOp::Compose { operations: new_ops }
            }
        }
    }

    /// Generate WGSL code
    /// time_offset is the cumulative time offset from parent Seqs (in seconds)
    pub fn to_wgsl(&self, time_offset: f64) -> String {
        match self {
            VisualOp::Simple {
                direction,
                x_mul,
                x_add,
                y_mul,
                y_add,
                z_mul,
                z_add,
                scale_mul,
                scale_add,
                velocity_mul,
                velocity_add,
                bend,
                alpha_set,
                alpha_mul,
                length,
            } => {
                let mut lines = Vec::new();

                // Check if this is a standalone Lm (only length is set, nothing else)
                let is_standalone_lm = direction.is_none()
                    && x_mul.is_none()
                    && x_add.is_none()
                    && y_mul.is_none()
                    && y_add.is_none()
                    && z_mul.is_none()
                    && z_add.is_none()
                    && scale_mul.is_none()
                    && scale_add.is_none()
                    && velocity_mul.is_none()
                    && velocity_add.is_none()
                    && bend.is_none()
                    && alpha_set.is_none()
                    && alpha_mul.is_none()
                    && *length != Rational64::new(1, 1);

                if is_standalone_lm {
                    // Standalone Lm: modify the runtime seg_length variable
                    lines.push(format!("seg_length = seg_length * {:.6};", rational_to_f32(*length)));
                    return lines.join("\n");
                }

                // Direction with optional Bend (cubic Bézier curve)
                if let Some((dx, dy, dz)) = direction {
                    if let Some((bx, by, bz, k)) = bend {
                        // Bend: create a symmetric cubic Bézier curve that bulges toward bend vector
                        // Note: if bend is parallel to dir, perp_len will be ~0 and we fall back to linear
                        lines.push(format!(
                            r#"// Bézier curve with bend (symmetric bulge)
direction = normalize(vec3<f32>({}, {}, {}));
let bend_raw = vec3<f32>({}, {}, {});
let k = {:.6};
let L = time * velocity;
// Remove component of bend along direction to get perpendicular
let bend_perp = bend_raw - dot(bend_raw, direction) * direction;
let perp_len = length(bend_perp);
// If bend is parallel to dir, perp is zero - use linear path instead
var offset = vec3<f32>(0.0, 0.0, 0.0);
if (perp_len > 0.001) {{
    let b_perp = bend_perp / perp_len;
    offset = b_perp * k * L * 0.5;
}}
// Bézier control points: both have same offset (symmetric bulge)
let p0 = vec3<f32>(0.0, 0.0, 0.0);
let p3 = direction * L;
let p1 = direction * (L / 3.0) + offset;
let p2 = direction * (L * 2.0 / 3.0) + offset;
// At t=1, position is p3
let pos = p3;
x += pos.x;
y += pos.y;
z += pos.z;"#,
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz),
                            rational_to_f32(*bx),
                            rational_to_f32(*by),
                            rational_to_f32(*bz),
                            rational_to_f32(*k)
                        ));
                    } else {
                        // No bend: simple linear movement
                        lines.push(format!(
                            "direction = normalize(vec3<f32>({}, {}, {}));",
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz)
                        ));
                        // Apply movement based on time using runtime velocity
                        lines.push("x += direction.x * time * velocity;".to_string());
                        lines.push("y += direction.y * time * velocity;".to_string());
                        lines.push("z += direction.z * time * velocity;".to_string());
                    }
                }

                if let Some(v) = x_mul {
                    lines.push(format!("x = x * {};", rational_to_f32(*v)));
                }
                if let Some(v) = x_add {
                    lines.push(format!("x = x + {};", rational_to_f32(*v)));
                }
                if let Some(v) = y_mul {
                    lines.push(format!("y = y * {};", rational_to_f32(*v)));
                }
                if let Some(v) = y_add {
                    lines.push(format!("y = y + {};", rational_to_f32(*v)));
                }
                if let Some(v) = z_mul {
                    lines.push(format!("z = z * {};", rational_to_f32(*v)));
                }
                if let Some(v) = z_add {
                    lines.push(format!("z = z + {};", rational_to_f32(*v)));
                }
                if let Some(v) = scale_mul {
                    lines.push(format!("scale = scale * {};", rational_to_f32(*v)));
                }
                if let Some(v) = scale_add {
                    lines.push(format!("scale = scale + {};", rational_to_f32(*v)));
                }
                if let Some(v) = velocity_mul {
                    lines.push(format!("velocity = velocity * {};", rational_to_f32(*v)));
                }
                if let Some(v) = velocity_add {
                    lines.push(format!("velocity = velocity + {};", rational_to_f32(*v)));
                }
                if let Some(v) = alpha_set {
                    lines.push(format!("alpha = {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = alpha_mul {
                    lines.push(format!("alpha = alpha * {:.6};", rational_to_f32(*v)));
                }

                lines.join("\n")
            }

            VisualOp::Seq { items } => {
                let mut wgsl = String::new();

                // Seq creates its own context - save parent's seg_length
                wgsl.push_str("// Seq context\n{\n");
                wgsl.push_str("    let parent_seg_length = seg_length;\n");

                // Calculate base times (compile-time) - will be scaled by seg_length at runtime
                let mut base_times: Vec<(f64, f64)> = Vec::new();
                let mut current_base_time = time_offset;

                for op in items.iter() {
                    let base_duration = rational_to_f64(op.length());
                    base_times.push((current_base_time, current_base_time + base_duration));
                    current_base_time += base_duration;
                }

                for (i, op) in items.iter().enumerate() {
                    let (base_start, base_end) = base_times[i];

                    // Generate segment block - times are scaled by seg_length at runtime
                    wgsl.push_str(&format!("    // Segment {}\n    {{\n", i));
                    wgsl.push_str(&op.to_wgsl_segment_runtime(base_start, base_end));
                    wgsl.push_str("\n    }\n");
                }

                // Continue in the last direction after Seq ends (using runtime velocity)
                if let Some(last_dir) = items.last().and_then(|op| op.last_direction()) {
                    let base_seq_end = current_base_time;
                    wgsl.push_str(&format!(
                        "    // Continue after Seq\n    {{\n        let seq_end = {:.6} * seg_length;\n        if (time > seq_end) {{\n            let dt = time - seq_end;\n            let dir = normalize(vec3<f32>({}, {}, {}));\n            x += dir.x * dt * velocity;\n            y += dir.y * dt * velocity;\n            z += dir.z * dt * velocity;\n        }}\n    }}\n",
                        base_seq_end,
                        rational_to_f32(last_dir.0),
                        rational_to_f32(last_dir.1),
                        rational_to_f32(last_dir.2)
                    ));
                }

                // Restore parent context (seg_length unchanged - Seq is its own context)
                wgsl.push_str("}\n");

                wgsl
            }

            VisualOp::Compose { operations } => {
                // Generate all operations in sequence
                operations
                    .iter()
                    .map(|op| op.to_wgsl(time_offset))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    // Constructor helpers for grammar
    fn simple_default() -> Self {
        VisualOp::Simple {
            x_mul: None,
            x_add: None,
            y_mul: None,
            y_add: None,
            z_mul: None,
            z_add: None,
            direction: None,
            scale_mul: None,
            scale_add: None,
            velocity_mul: None,
            velocity_add: None,
            bend: None,
            alpha_set: None,
            alpha_mul: None,
            length: Rational64::new(1, 1),
        }
    }

    pub fn direction(x: Rational64, y: Rational64, z: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { direction: ref mut d, .. } = op {
            *d = Some((x, y, z));
        }
        op
    }

    pub fn xm(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { x_mul: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn xa(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { x_add: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn ym(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { y_mul: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn ya(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { y_add: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn zm(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { z_mul: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn za(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { z_add: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn sm(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { scale_mul: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn sa(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { scale_add: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn vm(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { velocity_mul: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn va(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { velocity_add: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    pub fn lm(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { length: ref mut f, .. } = op {
            *f = v;
        }
        op
    }

    /// Create a Bend operation
    /// bx, by, bz: bend direction vector (will be made perpendicular to Direction)
    /// k: curvature strength (positive = toward bend vector, negative = away)
    pub fn bend(bx: Rational64, by: Rational64, bz: Rational64, k: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { bend: ref mut b, .. } = op {
            *b = Some((bx, by, bz, k));
        }
        op
    }

    /// Set alpha directly (Alpha 0 = invisible, Alpha 1 = visible)
    pub fn alpha(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { alpha_set: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    /// Multiply alpha (Am 0.5 = fade to 50%)
    pub fn am(v: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { alpha_mul: ref mut f, .. } = op {
            *f = Some(v);
        }
        op
    }

    /// Generate WGSL code for a time segment that accumulates position
    /// seg_start: when this segment begins (seconds)
    /// seg_end: when this segment ends (seconds)
    fn to_wgsl_segment(&self, seg_start: f64, seg_end: f64) -> String {
        match self {
            VisualOp::Simple {
                direction,
                x_mul,
                x_add,
                y_mul,
                y_add,
                z_mul,
                z_add,
                scale_mul,
                scale_add,
                velocity_mul,
                velocity_add,
                bend,
                alpha_set,
                alpha_mul,
                ..
            } => {
                let duration = seg_end - seg_start;
                let mut lines = Vec::new();

                lines.push(format!("    let seg_start = {:.6};", seg_start));
                lines.push(format!("    let seg_end = {:.6};", seg_end));
                lines.push(format!("    let duration = {:.6};", duration));

                // Calculate dt based on time position relative to segment
                lines.push(
                    r#"    var dt: f32 = 0.0;
    if (time >= seg_end) {
        dt = duration;
    } else if (time >= seg_start) {
        dt = time - seg_start;
    }
    let progress = dt / duration;"#
                        .to_string(),
                );

                // Direction with optional Bend (cubic Bézier curve)
                if let Some((dx, dy, dz)) = direction {
                    if let Some((bx, by, bz, k)) = bend {
                        // Bend: create a symmetric cubic Bézier curve that bulges toward bend vector
                        // Start and end direction remain the same
                        // L is the full segment length (not dependent on dt) - this ensures
                        // smooth transitions between segments
                        // Note: if bend is parallel to dir, perp_len will be ~0 and we fall back to linear
                        lines.push(format!(
                            r#"    // Bézier curve with bend (symmetric bulge)
    let dir = normalize(vec3<f32>({}, {}, {}));
    let bend_raw = vec3<f32>({}, {}, {});
    let k = {:.6};
    let L = duration * velocity;  // Full segment length for consistent curve shape
    // Remove component of bend along dir to get perpendicular
    let bend_perp = bend_raw - dot(bend_raw, dir) * dir;
    let perp_len = length(bend_perp);
    // If bend is parallel to dir, perp is zero - use linear path instead
    var offset = vec3<f32>(0.0, 0.0, 0.0);
    if (perp_len > 0.001) {{
        let b_perp = bend_perp / perp_len;
        offset = b_perp * k * L * 0.5;
    }}
    // Bézier control points: both P1 and P2 have same offset (symmetric bulge)
    let p0 = vec3<f32>(0.0, 0.0, 0.0);
    let p3 = dir * L;
    let p1 = dir * (L / 3.0) + offset;
    let p2 = dir * (L * 2.0 / 3.0) + offset;
    // Cubic Bézier: (1-t)³P0 + 3(1-t)²tP1 + 3(1-t)t²P2 + t³P3
    let t = progress;  // progress = dt/duration, so at dt=0, t=0; at dt=duration, t=1
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;
    let pos = mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3;
    x += pos.x;
    y += pos.y;
    z += pos.z;"#,
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz),
                            rational_to_f32(*bx),
                            rational_to_f32(*by),
                            rational_to_f32(*bz),
                            rational_to_f32(*k)
                        ));
                    } else {
                        // No bend: simple linear displacement
                        lines.push(format!(
                            "    let dir = normalize(vec3<f32>({}, {}, {}));\n    x += dir.x * dt * velocity;\n    y += dir.y * dt * velocity;\n    z += dir.z * dt * velocity;",
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz)
                        ));
                    }
                }

                // Collect ops that should only apply when we've reached this segment
                let mut segment_ops = Vec::new();

                // Multiply ops: set at segment start
                if let Some(v) = x_mul {
                    segment_ops.push(format!("        x = x * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = y_mul {
                    segment_ops.push(format!("        y = y * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = z_mul {
                    segment_ops.push(format!("        z = z * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = scale_mul {
                    segment_ops.push(format!("        scale = scale * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = velocity_mul {
                    segment_ops.push(format!("        velocity = velocity * {:.6};", rational_to_f32(*v)));
                }

                // Add ops: set at segment start
                if let Some(v) = x_add {
                    segment_ops.push(format!("        x = x + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = y_add {
                    segment_ops.push(format!("        y = y + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = z_add {
                    segment_ops.push(format!("        z = z + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = scale_add {
                    segment_ops.push(format!("        scale = scale + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = velocity_add {
                    segment_ops.push(format!("        velocity = velocity + {:.6};", rational_to_f32(*v)));
                }

                // Alpha: set directly or multiply
                if let Some(v) = alpha_set {
                    segment_ops.push(format!("        alpha = {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = alpha_mul {
                    segment_ops.push(format!("        alpha = alpha * {:.6};", rational_to_f32(*v)));
                }

                // Wrap ops in time check - only apply when we've reached this segment
                if !segment_ops.is_empty() {
                    lines.push("    if (time >= seg_start) {".to_string());
                    for op in segment_ops {
                        lines.push(op);
                    }
                    lines.push("    }".to_string());
                }

                lines.join("\n")
            }

            VisualOp::Seq { items } => {
                // Nested Seq: recursively generate inner segments
                let mut wgsl = String::new();
                let mut inner_time = seg_start;
                let total_len = rational_to_f64(self.length());
                let outer_duration = seg_end - seg_start;

                for op in items {
                    let inner_duration = rational_to_f64(op.length());
                    let scaled = if total_len > 0.0 {
                        (inner_duration / total_len) * outer_duration
                    } else {
                        0.0
                    };
                    wgsl.push_str(&op.to_wgsl_segment(inner_time, inner_time + scaled));
                    wgsl.push('\n');
                    inner_time += scaled;
                }
                wgsl
            }

            VisualOp::Compose { operations } => {
                operations
                    .iter()
                    .map(|op| op.to_wgsl_segment(seg_start, seg_end))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    /// Generate WGSL code for a time segment using runtime seg_length
    /// base_start/base_end: compile-time segment times (will be scaled by seg_length at runtime)
    fn to_wgsl_segment_runtime(&self, base_start: f64, base_end: f64) -> String {
        match self {
            VisualOp::Simple {
                direction,
                x_mul,
                x_add,
                y_mul,
                y_add,
                z_mul,
                z_add,
                scale_mul,
                scale_add,
                velocity_mul,
                velocity_add,
                bend,
                alpha_set,
                alpha_mul,
                ..
            } => {
                let base_duration = base_end - base_start;
                let mut lines = Vec::new();

                // Times are scaled by seg_length at runtime
                lines.push(format!("        let seg_start = {:.6} * seg_length;", base_start));
                lines.push(format!("        let seg_end = {:.6} * seg_length;", base_end));
                lines.push(format!("        let duration = {:.6} * seg_length;", base_duration));

                // Calculate dt based on time position relative to segment
                lines.push(
                    r#"        var dt: f32 = 0.0;
        if (time >= seg_end) {
            dt = duration;
        } else if (time >= seg_start) {
            dt = time - seg_start;
        }
        let progress = select(0.0, dt / duration, duration > 0.0);"#
                        .to_string(),
                );

                // Direction with optional Bend (cubic Bézier curve)
                if let Some((dx, dy, dz)) = direction {
                    if let Some((bx, by, bz, k)) = bend {
                        lines.push(format!(
                            r#"        // Bézier curve with bend (symmetric bulge)
        let dir = normalize(vec3<f32>({}, {}, {}));
        let bend_raw = vec3<f32>({}, {}, {});
        let k = {:.6};
        let L = duration * velocity;
        let bend_perp = bend_raw - dot(bend_raw, dir) * dir;
        let perp_len = length(bend_perp);
        var offset = vec3<f32>(0.0, 0.0, 0.0);
        if (perp_len > 0.001) {{
            let b_perp = bend_perp / perp_len;
            offset = b_perp * k * L * 0.5;
        }}
        let p0 = vec3<f32>(0.0, 0.0, 0.0);
        let p3 = dir * L;
        let p1 = dir * (L / 3.0) + offset;
        let p2 = dir * (L * 2.0 / 3.0) + offset;
        let t = progress;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        let pos = mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3;
        x += pos.x;
        y += pos.y;
        z += pos.z;"#,
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz),
                            rational_to_f32(*bx),
                            rational_to_f32(*by),
                            rational_to_f32(*bz),
                            rational_to_f32(*k)
                        ));
                    } else {
                        // No bend: simple linear displacement
                        lines.push(format!(
                            "        let dir = normalize(vec3<f32>({}, {}, {}));\n        x += dir.x * dt * velocity;\n        y += dir.y * dt * velocity;\n        z += dir.z * dt * velocity;",
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz)
                        ));
                    }
                }

                // Collect ops that should only apply when we've reached this segment
                let mut segment_ops = Vec::new();

                if let Some(v) = x_mul {
                    segment_ops.push(format!("            x = x * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = y_mul {
                    segment_ops.push(format!("            y = y * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = z_mul {
                    segment_ops.push(format!("            z = z * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = scale_mul {
                    segment_ops.push(format!("            scale = scale * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = velocity_mul {
                    segment_ops.push(format!("            velocity = velocity * {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = x_add {
                    segment_ops.push(format!("            x = x + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = y_add {
                    segment_ops.push(format!("            y = y + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = z_add {
                    segment_ops.push(format!("            z = z + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = scale_add {
                    segment_ops.push(format!("            scale = scale + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = velocity_add {
                    segment_ops.push(format!("            velocity = velocity + {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = alpha_set {
                    segment_ops.push(format!("            alpha = {:.6};", rational_to_f32(*v)));
                }
                if let Some(v) = alpha_mul {
                    segment_ops.push(format!("            alpha = alpha * {:.6};", rational_to_f32(*v)));
                }

                // Wrap ops in time check - only apply when we've reached this segment
                if !segment_ops.is_empty() {
                    lines.push("        if (time >= seg_start) {".to_string());
                    for op in segment_ops {
                        lines.push(op);
                    }
                    lines.push("        }".to_string());
                }

                lines.join("\n")
            }

            VisualOp::Seq { items } => {
                // Nested Seq: recursively generate inner segments
                let mut wgsl = String::new();
                let mut inner_base_time = base_start;
                let total_len = rational_to_f64(self.length());
                let outer_base_duration = base_end - base_start;

                for op in items {
                    let inner_duration = rational_to_f64(op.length());
                    let scaled = if total_len > 0.0 {
                        (inner_duration / total_len) * outer_base_duration
                    } else {
                        0.0
                    };
                    wgsl.push_str(&op.to_wgsl_segment_runtime(inner_base_time, inner_base_time + scaled));
                    wgsl.push('\n');
                    inner_base_time += scaled;
                }
                wgsl
            }

            VisualOp::Compose { operations } => {
                operations
                    .iter()
                    .map(|op| op.to_wgsl_segment_runtime(base_start, base_end))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

// Helper functions for composition
fn compose_mul(a: Option<Rational64>, b: Option<Rational64>) -> Option<Rational64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a * b),
        (a, b) => a.or(b),
    }
}

fn compose_add(a: Option<Rational64>, b: Option<Rational64>) -> Option<Rational64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a + b),
        (a, b) => a.or(b),
    }
}

fn rational_to_f32(r: Rational64) -> f32 {
    *r.numer() as f32 / *r.denom() as f32
}

fn rational_to_f64(r: Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

// Helper to convert float to rational (for DSL parsing)
pub fn float_to_rational(f: f64) -> Rational64 {
    // Use a reasonable precision (1000000 denominator max)
    let precision = 1_000_000i64;
    let numer = (f * precision as f64).round() as i64;
    Rational64::new(numer, precision)
}

// ============================================================================

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
fn normalize(v: vec3<f32>) -> vec3<f32> { return v; }
fn length(v: vec3<f32>) -> f32 { return 1.0; }
fn select(a: f32, b: f32, c: bool) -> f32 { if c { return b; } else { return a; } }

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
    var seg_length: f32 = 1.0;  // Duration multiplier for Seq segments
    var direction: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
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

    #[test]
    fn test_visual_op_direction() {
        let op = VisualOp::direction(
            Rational64::new(0, 1),
            Rational64::new(-1, 1),
            Rational64::new(0, 1),
        );
        let wgsl = op.to_wgsl(0.0);
        assert!(wgsl.contains("direction = normalize"));
        // Direction should apply movement using runtime velocity
        assert!(wgsl.contains("x += direction.x * time * velocity"));
        assert!(wgsl.contains("y += direction.y * time * velocity"));
        assert!(wgsl.contains("z += direction.z * time * velocity"));
    }

    #[test]
    fn test_visual_op_compose() {
        let dir = VisualOp::direction(
            Rational64::new(0, 1),
            Rational64::new(-1, 1),
            Rational64::new(0, 1),
        );
        let vm = VisualOp::vm(Rational64::new(2, 1));
        let composed = dir.compose(vm);
        let wgsl = composed.to_wgsl(0.0);
        assert!(wgsl.contains("direction = normalize"));
        assert!(wgsl.contains("velocity = velocity * 2"));
    }

    #[test]
    fn test_visual_op_seq_accumulating() {
        let dir1 = VisualOp::direction(
            Rational64::new(0, 1),
            Rational64::new(-1, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(2, 1)));

        let dir2 = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seq = VisualOp::Seq { items: vec![dir1, dir2] };
        let wgsl = seq.to_wgsl(0.0);

        // Should have two separate segment blocks (not if/else)
        assert!(wgsl.contains("// Segment 0"));
        assert!(wgsl.contains("// Segment 1"));

        // Should have accumulating pattern: check time against seg_end
        assert!(wgsl.contains("if (time >= seg_end)"));
        assert!(wgsl.contains("else if (time >= seg_start)"));

        // Should use runtime velocity (no modifier baked in)
        assert!(wgsl.contains("x += dir.x * dt * velocity"));
        assert!(wgsl.contains("y += dir.y * dt * velocity"));

        // Segment boundaries should be correct
        assert!(wgsl.contains("seg_end = 2.000000")); // First segment ends at 2
        assert!(wgsl.contains("seg_start = 2.000000")); // Second segment starts at 2
        assert!(wgsl.contains("seg_end = 3.000000")); // Second segment ends at 3
    }

    #[test]
    fn test_visual_op_seq_with_multiply_ops() {
        // Test that multiply ops are set when segment is reached
        let xm = VisualOp::xm(Rational64::new(2, 1))
            .compose(VisualOp::lm(Rational64::new(1, 1)));

        let seq = VisualOp::Seq { items: vec![xm] };
        let wgsl = seq.to_wgsl(0.0);

        // Should be wrapped in time check and set immediately
        assert!(wgsl.contains("if (time >= seg_start)"));
        assert!(wgsl.contains("x = x * 2"));
    }

    #[test]
    fn test_visual_op_seq_with_add_ops() {
        // Test that add ops are set when segment is reached
        let ya = VisualOp::ya(Rational64::new(5, 1))
            .compose(VisualOp::lm(Rational64::new(2, 1)));

        let seq = VisualOp::Seq { items: vec![ya] };
        let wgsl = seq.to_wgsl(0.0);

        // Should be wrapped in time check and set immediately
        assert!(wgsl.contains("if (time >= seg_start)"));
        assert!(wgsl.contains("y = y + 5"));
    }

    #[test]
    fn test_visual_op_bend() {
        // Test Bend creates Bézier curve code
        let dir = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        );
        let bend = VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 2), // k = 0.5
        );
        let composed = dir.compose(bend);
        let wgsl = composed.to_wgsl(0.0);

        // Should have Bézier curve components
        assert!(wgsl.contains("Bézier"));
        assert!(wgsl.contains("b_perp"));
        assert!(wgsl.contains("bend_raw"));
        assert!(wgsl.contains("offset"));
    }

    #[test]
    fn test_visual_op_bend_in_seq() {
        // Test Bend works inside Seq with proper Bézier evaluation
        let segment = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 2),
        )).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seq = VisualOp::Seq { items: vec![segment] };
        let wgsl = seq.to_wgsl(0.0);

        // Should have Bézier curve with time interpolation
        assert!(wgsl.contains("Bézier"), "Output should mention Bézier");
        assert!(wgsl.contains("mt3"), "Output should have (1-t)^3 component"); // (1-t)^3 component
        assert!(wgsl.contains("t3"), "Output should have t^3 component");  // t^3 component
    }

    #[test]
    fn test_seq_with_three_bends_and_am() {
        // Reproduce user's case: 3 Direction+Bend segments followed by Am 0.0
        let seg1 = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 1),
        )).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seg2 = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(-1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 1),
        )).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seg3 = VisualOp::direction(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(-1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 1),
        )).compose(VisualOp::lm(Rational64::new(1, 1)));

        let am_seg = VisualOp::am(Rational64::new(0, 1));

        let seq = VisualOp::Seq { items: vec![seg1, seg2, seg3, am_seg] };
        let wgsl = seq.to_wgsl(0.0);

        println!("Generated WGSL:\n{}", wgsl);

        // Should have 4 segments
        assert!(wgsl.contains("// Segment 0"));
        assert!(wgsl.contains("// Segment 1"));
        assert!(wgsl.contains("// Segment 2"));
        assert!(wgsl.contains("// Segment 3"));

        // Verify it compiles
        assert!(validate_wgsl(&wgsl).is_ok(), "Generated WGSL should be valid");
    }

    #[test]
    fn test_standalone_lm_generates_seg_length() {
        // Standalone Lm should generate seg_length modification
        let lm = VisualOp::lm(Rational64::new(1, 2));
        let wgsl = lm.to_wgsl(0.0);
        println!("Standalone Lm 1/2:\n{}", wgsl);
        assert!(wgsl.contains("seg_length = seg_length * 0.5"), "Should modify seg_length");
    }

    #[test]
    fn test_seq_uses_seg_length_runtime() {
        // Seq should use seg_length to scale segment times at runtime
        let seg1 = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seq = VisualOp::Seq { items: vec![seg1] };
        let wgsl = seq.to_wgsl(0.0);
        println!("Seq with runtime seg_length:\n{}", wgsl);

        // Should use seg_length to scale times at runtime
        assert!(wgsl.contains("* seg_length"), "Segment times should be scaled by seg_length");
        assert!(wgsl.contains("parent_seg_length"), "Should save parent context");
    }

    #[test]
    fn test_lm_before_seq_scales_segments() {
        // Test that Lm 1/4; Seq[...] generates both seg_length modification and Seq
        // This simulates: Lm 1/4; Seq [Direction | Lm 1, Direction | Lm 1]

        // First generate the standalone Lm
        let lm = VisualOp::lm(Rational64::new(1, 4));
        let lm_wgsl = lm.to_wgsl(0.0);

        // Then generate the Seq
        let seg1 = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seg2 = VisualOp::direction(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seq = VisualOp::Seq { items: vec![seg1, seg2] };
        let seq_wgsl = seq.to_wgsl(0.0);

        // Combined output
        let wgsl = format!("{}\n{}", lm_wgsl, seq_wgsl);
        println!("Lm 1/4 + Seq:\n{}", wgsl);

        // Lm should set seg_length
        assert!(wgsl.contains("seg_length = seg_length * 0.25"), "Lm should modify seg_length");

        // Seq should use seg_length for runtime scaling
        assert!(wgsl.contains("1.000000 * seg_length"), "Segment times should be scaled by seg_length");
    }
} 

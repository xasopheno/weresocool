use std::collections::HashMap;
use std::ops::Mul;
use naga::front::wgsl::Frontend;
use colored::*;
use num_rational::Rational64;

pub const MAX_STEPS: u32 = 4; // compile-time bound for recipe size

// ============================================================================
// WgslValue - can be a compile-time rational or a runtime WGSL expression
// ============================================================================

/// A value that can be either a compile-time rational number or a WGSL expression string
#[derive(Clone, Debug, PartialEq)]
pub enum WgslValue {
    /// Compile-time rational number (e.g., 1/2, 3, 0.5)
    Rational(Rational64),
    /// Runtime WGSL expression (e.g., "x * time", "sin(y)")
    Expr(String),
}

impl WgslValue {
    /// Convert to WGSL code string
    pub fn to_wgsl(&self) -> String {
        match self {
            WgslValue::Rational(r) => format!("{:.6}", rational_to_f32(*r)),
            WgslValue::Expr(s) => s.clone(),
        }
    }

    /// Check if this is a compile-time constant
    pub fn is_constant(&self) -> bool {
        matches!(self, WgslValue::Rational(_))
    }

    /// Get as rational if it's a constant
    pub fn as_rational(&self) -> Option<Rational64> {
        match self {
            WgslValue::Rational(r) => Some(*r),
            WgslValue::Expr(_) => None,
        }
    }
}

impl From<Rational64> for WgslValue {
    fn from(r: Rational64) -> Self {
        WgslValue::Rational(r)
    }
}

impl From<String> for WgslValue {
    fn from(s: String) -> Self {
        WgslValue::Expr(s)
    }
}

impl From<&str> for WgslValue {
    fn from(s: &str) -> Self {
        WgslValue::Expr(s.to_string())
    }
}

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
        /// ArcTo: (target_dir_x, target_dir_y, target_dir_z, strength)
        /// Creates a curved path that ends pointing toward target_dir
        /// k=1 fully commits to new direction, k=0.5 is halfway, etc.
        arc_to: Option<(Rational64, Rational64, Rational64, Rational64)>,
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
    /// Raw WGSL code (passed through directly)
    Raw {
        wgsl: String,
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
            arc_to: None,
            alpha_set: None,
            alpha_mul: None,
            length: Rational64::new(1, 1),
        }
    }
}

// ============================================================================
// VisualPointOp - Normalized form for visual operations (analogous to PointOp)
// ============================================================================

/// Space warp functions that modify direction over time
#[derive(Clone, Debug, PartialEq)]
pub enum Warp {
    /// Bend: rotate direction around an axis, angle proportional to progress
    Bend {
        axis: (Rational64, Rational64, Rational64),
        strength: Rational64,
    },
    /// ArcTo: steer direction toward a target, angle proportional to progress
    ArcTo {
        target: (Rational64, Rational64, Rational64),
        strength: Rational64,
    },
}

/// Normalized visual operation - carries all state for a single time segment
#[derive(Clone, Debug, PartialEq)]
pub struct VisualPointOp {
    // Position transforms (multiplicative + additive)
    pub x_mul: Rational64,
    pub x_add: Rational64,
    pub y_mul: Rational64,
    pub y_add: Rational64,
    pub z_mul: Rational64,
    pub z_add: Rational64,

    // Direction (base direction before warps)
    pub direction: Option<(Rational64, Rational64, Rational64)>,

    // Scalar modifiers
    pub scale_mul: Rational64,
    pub scale_add: Rational64,
    pub velocity_mul: Rational64,
    pub velocity_add: Rational64,
    pub alpha_mul: Rational64,
    pub alpha_set: Option<Rational64>,

    // Space warps - CHAIN like audio filters
    pub warps: Vec<Warp>,

    // Duration
    pub length: Rational64,
}

impl Default for VisualPointOp {
    fn default() -> Self {
        VisualPointOp {
            x_mul: Rational64::new(1, 1),
            x_add: Rational64::new(0, 1),
            y_mul: Rational64::new(1, 1),
            y_add: Rational64::new(0, 1),
            z_mul: Rational64::new(1, 1),
            z_add: Rational64::new(0, 1),
            direction: None,
            scale_mul: Rational64::new(1, 1),
            scale_add: Rational64::new(0, 1),
            velocity_mul: Rational64::new(1, 1),
            velocity_add: Rational64::new(0, 1),
            alpha_mul: Rational64::new(1, 1),
            alpha_set: None,
            warps: Vec::new(),
            length: Rational64::new(1, 1),
        }
    }
}

/// Composition: multiply two VisualPointOps
/// - Multiplicative fields: multiply
/// - Additive fields: add
/// - Direction: right-biased (last wins)
/// - Warps: concatenate (chain like filters)
impl Mul for VisualPointOp {
    type Output = VisualPointOp;

    fn mul(self, other: VisualPointOp) -> VisualPointOp {
        VisualPointOp {
            // Multiplicative fields
            x_mul: self.x_mul * other.x_mul,
            y_mul: self.y_mul * other.y_mul,
            z_mul: self.z_mul * other.z_mul,
            scale_mul: self.scale_mul * other.scale_mul,
            velocity_mul: self.velocity_mul * other.velocity_mul,
            alpha_mul: self.alpha_mul * other.alpha_mul,

            // Additive fields
            x_add: self.x_add + other.x_add,
            y_add: self.y_add + other.y_add,
            z_add: self.z_add + other.z_add,
            scale_add: self.scale_add + other.scale_add,
            velocity_add: self.velocity_add + other.velocity_add,

            // Right-biased (last wins)
            direction: other.direction.or(self.direction),
            alpha_set: other.alpha_set.or(self.alpha_set),

            // Chain (concatenate)
            warps: [self.warps, other.warps].concat(),

            // Multiply lengths
            length: self.length * other.length,
        }
    }
}

// ============================================================================
// VisualNormalForm - Container for normalized operations
// ============================================================================

/// Normalized form for visual operations
#[derive(Clone, Debug, PartialEq)]
pub struct VisualNormalForm {
    /// Flat list of point operations (each represents a time segment)
    pub operations: Vec<VisualPointOp>,
    /// Total duration
    pub length: Rational64,
}

impl Default for VisualNormalForm {
    fn default() -> Self {
        VisualNormalForm {
            operations: vec![VisualPointOp::default()],
            length: Rational64::new(1, 1),
        }
    }
}

impl VisualNormalForm {
    /// Create an empty normal form
    pub fn empty() -> Self {
        VisualNormalForm {
            operations: Vec::new(),
            length: Rational64::new(0, 1),
        }
    }

    /// Apply a VisualPointOp to all operations in this normal form
    pub fn apply(&mut self, modifier: &VisualPointOp) {
        for op in &mut self.operations {
            *op = op.clone() * modifier.clone();
        }
        self.length = self.length * modifier.length;
    }

    /// Join two normal forms in sequence (like Seq)
    pub fn join_sequence(mut self, other: VisualNormalForm) -> VisualNormalForm {
        self.operations.extend(other.operations);
        self.length = self.length + other.length;
        self
    }

    /// Generate WGSL code from this normalized form
    pub fn to_wgsl(&self) -> String {
        let mut wgsl = String::new();
        let total_length = rational_to_f64(self.length);

        if self.operations.is_empty() {
            return wgsl;
        }

        // Single operation - no time gating needed
        if self.operations.len() == 1 {
            wgsl.push_str(&self.operations[0].to_wgsl_segment(0.0, total_length, total_length));
            return wgsl;
        }

        // Multiple operations - create time-gated segments
        wgsl.push_str("// Sequence of operations\n");

        let mut current_time = 0.0;
        for (i, op) in self.operations.iter().enumerate() {
            let seg_duration = rational_to_f64(op.length);
            let seg_end = current_time + seg_duration;

            if i == 0 {
                wgsl.push_str(&format!(
                    "if (time < {:.6}) {{\n",
                    seg_end
                ));
            } else if i == self.operations.len() - 1 {
                wgsl.push_str(&format!(
                    "}} else {{\n"
                ));
            } else {
                wgsl.push_str(&format!(
                    "}} else if (time < {:.6}) {{\n",
                    seg_end
                ));
            }

            wgsl.push_str(&op.to_wgsl_segment(current_time, seg_end, total_length));
            current_time = seg_end;
        }

        wgsl.push_str("}\n");
        wgsl
    }
}

impl VisualPointOp {
    /// Generate WGSL for a single segment
    fn to_wgsl_segment(&self, seg_start: f64, seg_end: f64, total_length: f64) -> String {
        let mut lines = Vec::new();
        let seg_duration = seg_end - seg_start;

        // Calculate progress within this segment
        lines.push(format!(
            "    let seg_start = {:.6};",
            seg_start
        ));
        lines.push(format!(
            "    let seg_duration = {:.6};",
            seg_duration
        ));
        lines.push("    let local_time = time - seg_start;".to_string());
        lines.push("    let progress = local_time / seg_duration;".to_string());

        // Set direction if specified
        if let Some((dx, dy, dz)) = self.direction {
            lines.push(format!(
                "    direction = normalize(vec3<f32>({:.6}, {:.6}, {:.6}));",
                rational_to_f32(dx),
                rational_to_f32(dy),
                rational_to_f32(dz)
            ));
        }

        // Apply warps to direction (this is the key part!)
        for warp in &self.warps {
            match warp {
                Warp::Bend { axis: (ax, ay, az), strength } => {
                    lines.push(format!(
                        r#"    // Bend warp: rotate direction around axis
    {{
        let bend_axis = normalize(vec3<f32>({:.6}, {:.6}, {:.6}));
        let bend_k = {:.6};
        let angle = bend_k * progress * 1.5708;  // k * progress * π/2
        let cos_a = cos(angle);
        let sin_a = sin(angle);
        // Rodrigues' rotation formula
        direction = direction * cos_a
                  + cross(bend_axis, direction) * sin_a
                  + bend_axis * dot(bend_axis, direction) * (1.0 - cos_a);
    }}"#,
                        rational_to_f32(*ax),
                        rational_to_f32(*ay),
                        rational_to_f32(*az),
                        rational_to_f32(*strength)
                    ));
                }
                Warp::ArcTo { target: (tx, ty, tz), strength } => {
                    lines.push(format!(
                        r#"    // ArcTo warp: steer direction toward target
    {{
        let target_dir = normalize(vec3<f32>({:.6}, {:.6}, {:.6}));
        let arc_k = {:.6};
        let rot_axis_raw = cross(direction, target_dir);
        let rot_axis_len = length(rot_axis_raw);
        if (rot_axis_len > 0.001) {{
            let rot_axis = rot_axis_raw / rot_axis_len;
            let max_angle = acos(clamp(dot(direction, target_dir), -1.0, 1.0));
            let angle = arc_k * progress * max_angle;
            let cos_a = cos(angle);
            let sin_a = sin(angle);
            // Rodrigues' rotation formula
            direction = direction * cos_a
                      + cross(rot_axis, direction) * sin_a
                      + rot_axis * dot(rot_axis, direction) * (1.0 - cos_a);
        }}
    }}"#,
                        rational_to_f32(*tx),
                        rational_to_f32(*ty),
                        rational_to_f32(*tz),
                        rational_to_f32(*strength)
                    ));
                }
            }
        }

        // Move using direction
        if self.direction.is_some() || !self.warps.is_empty() {
            lines.push("    // Move along direction".to_string());
            lines.push("    x += direction.x * local_time * velocity;".to_string());
            lines.push("    y += direction.y * local_time * velocity;".to_string());
            lines.push("    z += direction.z * local_time * velocity;".to_string());
        }

        // Apply scalar modifiers
        if self.x_mul != Rational64::new(1, 1) {
            lines.push(format!("    x = x * {:.6};", rational_to_f32(self.x_mul)));
        }
        if self.x_add != Rational64::new(0, 1) {
            lines.push(format!("    x = x + {:.6};", rational_to_f32(self.x_add)));
        }
        if self.y_mul != Rational64::new(1, 1) {
            lines.push(format!("    y = y * {:.6};", rational_to_f32(self.y_mul)));
        }
        if self.y_add != Rational64::new(0, 1) {
            lines.push(format!("    y = y + {:.6};", rational_to_f32(self.y_add)));
        }
        if self.z_mul != Rational64::new(1, 1) {
            lines.push(format!("    z = z * {:.6};", rational_to_f32(self.z_mul)));
        }
        if self.z_add != Rational64::new(0, 1) {
            lines.push(format!("    z = z + {:.6};", rational_to_f32(self.z_add)));
        }
        if self.scale_mul != Rational64::new(1, 1) {
            lines.push(format!("    scale = scale * {:.6};", rational_to_f32(self.scale_mul)));
        }
        if self.scale_add != Rational64::new(0, 1) {
            lines.push(format!("    scale = scale + {:.6};", rational_to_f32(self.scale_add)));
        }
        if self.velocity_mul != Rational64::new(1, 1) {
            lines.push(format!("    velocity = velocity * {:.6};", rational_to_f32(self.velocity_mul)));
        }
        if self.velocity_add != Rational64::new(0, 1) {
            lines.push(format!("    velocity = velocity + {:.6};", rational_to_f32(self.velocity_add)));
        }
        if let Some(alpha) = self.alpha_set {
            lines.push(format!("    alpha = {:.6};", rational_to_f32(alpha)));
        }
        if self.alpha_mul != Rational64::new(1, 1) {
            lines.push(format!("    alpha = alpha * {:.6};", rational_to_f32(self.alpha_mul)));
        }

        lines.join("\n") + "\n"
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
            // Raw WGSL has no duration concept - use default of 1
            VisualOp::Raw { .. } => Rational64::new(1, 1),
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
            // Raw WGSL has no direction
            VisualOp::Raw { .. } => None,
        }
    }

    /// Convert this VisualOp to a VisualNormalForm
    /// This flattens the AST into a list of VisualPointOps that can generate WGSL
    pub fn normalize(&self) -> VisualNormalForm {
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
                arc_to,
                alpha_set,
                alpha_mul,
                length,
            } => {
                // Convert Simple to a single VisualPointOp
                let mut warps = Vec::new();

                // Extract bend/arc_to as warps (they apply as path modifiers)
                if let Some((bx, by, bz, k)) = bend {
                    warps.push(Warp::Bend {
                        axis: (*bx, *by, *bz),
                        strength: *k,
                    });
                }
                if let Some((tx, ty, tz, k)) = arc_to {
                    warps.push(Warp::ArcTo {
                        target: (*tx, *ty, *tz),
                        strength: *k,
                    });
                }

                let point_op = VisualPointOp {
                    x_mul: x_mul.unwrap_or_else(|| Rational64::new(1, 1)),
                    x_add: x_add.unwrap_or_else(|| Rational64::new(0, 1)),
                    y_mul: y_mul.unwrap_or_else(|| Rational64::new(1, 1)),
                    y_add: y_add.unwrap_or_else(|| Rational64::new(0, 1)),
                    z_mul: z_mul.unwrap_or_else(|| Rational64::new(1, 1)),
                    z_add: z_add.unwrap_or_else(|| Rational64::new(0, 1)),
                    direction: *direction,
                    scale_mul: scale_mul.unwrap_or_else(|| Rational64::new(1, 1)),
                    scale_add: scale_add.unwrap_or_else(|| Rational64::new(0, 1)),
                    velocity_mul: velocity_mul.unwrap_or_else(|| Rational64::new(1, 1)),
                    velocity_add: velocity_add.unwrap_or_else(|| Rational64::new(0, 1)),
                    alpha_mul: alpha_mul.unwrap_or_else(|| Rational64::new(1, 1)),
                    alpha_set: *alpha_set,
                    warps,
                    length: *length,
                };

                VisualNormalForm {
                    operations: vec![point_op],
                    length: *length,
                }
            }

            VisualOp::Seq { items } => {
                // Seq: join all items in sequence
                let mut result = VisualNormalForm::empty();
                for item in items {
                    let item_nf = item.normalize();
                    result = result.join_sequence(item_nf);
                }
                result
            }

            VisualOp::Compose { operations } => {
                // Compose: apply each operation in order
                // Start with the first, then apply the rest as modifiers
                if operations.is_empty() {
                    return VisualNormalForm::default();
                }

                let mut result = operations[0].normalize();

                for op in &operations[1..] {
                    let modifier_nf = op.normalize();
                    // Apply each modifier's PointOps to the result
                    // For a single-PointOp modifier (like Bend alone), apply to all
                    if modifier_nf.operations.len() == 1 {
                        result.apply(&modifier_nf.operations[0]);
                    } else {
                        // For multi-PointOp modifiers, this is more complex
                        // For now, apply each modifier op to corresponding result op
                        for (i, mod_op) in modifier_nf.operations.iter().enumerate() {
                            if i < result.operations.len() {
                                result.operations[i] = result.operations[i].clone() * mod_op.clone();
                            }
                        }
                    }
                }

                result
            }

            VisualOp::Raw { .. } => {
                // Raw WGSL can't be normalized - return empty
                // (Raw is only used for pass-through code)
                VisualNormalForm::default()
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
                arc_to,
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
                arc_to,
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
    ///
    /// Composition semantics (order matters for match arms):
    /// - Seq | Seq → distribute left Seq into each right Seq item
    /// - Seq | Simple → distribute Simple into each Seq item
    /// - Simple | Simple → merge fields
    /// - Simple | Seq → Compose { [Simple, Seq] } (Simple runs BEFORE Seq, NOT distributed)
    /// - Compose | Seq → compose last item with Seq (enables recursive Seq | Seq)
    /// - Compose | anything → compose last item with other
    /// - anything | Compose → prepend self to Compose
    pub fn compose(self, other: VisualOp) -> VisualOp {
        match (self, other) {
            // Seq | Seq → distribute left into each item of right (like WereSoCool)
            (seq1 @ VisualOp::Seq { .. }, VisualOp::Seq { items: items2 }) => {
                let distributed_items = items2
                    .into_iter()
                    .map(|item| seq1.clone().compose(item))
                    .collect();
                VisualOp::Seq { items: distributed_items }
            }

            // Seq | Simple → distribute Simple into each Seq item
            // This means Seq [A, B] | Vm 1 becomes Seq [A | Vm 1, B | Vm 1]
            (VisualOp::Seq { items }, simple @ VisualOp::Simple { .. }) => {
                let distributed_items = items
                    .into_iter()
                    .map(|item| item.compose(simple.clone()))
                    .collect();
                VisualOp::Seq { items: distributed_items }
            }

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
                    arc_to: arc_to1,
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
                    arc_to: arc_to2,
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
                arc_to: arc_to2.or(arc_to1), // Later wins
                alpha_set: alpha_set2.or(alpha_set1), // Later wins
                alpha_mul: compose_mul(alpha_mul1, alpha_mul2),
                length: len1 * len2, // Multiply lengths
            },

            // Simple | Seq → apply Simple's modifiers before Seq runs
            // This creates Compose { [Simple, Seq] } so Simple runs first (NOT distributed!)
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
                    arc_to,
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
                    arc_to,
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
                    || arc_to.is_some()
                    || alpha_set.is_some()
                    || alpha_mul.is_some();

                if has_fields {
                    VisualOp::Compose {
                        operations: vec![simple_remainder, scaled_seq],
                    }
                } else {
                    scaled_seq
                }
            }

            // Compose | Seq → compose last item with Seq (enables recursive Seq | Seq)
            (VisualOp::Compose { mut operations }, right_seq @ VisualOp::Seq { .. }) => {
                if let Some(last) = operations.pop() {
                    let composed_last = last.compose(right_seq);
                    operations.push(composed_last);
                    if operations.len() == 1 {
                        operations.pop().unwrap()
                    } else {
                        VisualOp::Compose { operations }
                    }
                } else {
                    right_seq
                }
            }

            // Compose | anything → compose last item with other
            (VisualOp::Compose { mut operations }, other) => {
                if let Some(last) = operations.pop() {
                    let composed_last = last.compose(other);
                    operations.push(composed_last);
                    if operations.len() == 1 {
                        operations.pop().unwrap()
                    } else {
                        VisualOp::Compose { operations }
                    }
                } else {
                    other
                }
            }

            // anything | Compose → prepend self to Compose
            (other, VisualOp::Compose { operations }) => {
                let mut new_ops = vec![other];
                new_ops.extend(operations);
                VisualOp::Compose { operations: new_ops }
            }

            // Raw | anything or anything | Raw → wrap in Compose (Raw is pass-through)
            (raw @ VisualOp::Raw { .. }, other) => {
                VisualOp::Compose { operations: vec![raw, other] }
            }
            (other, raw @ VisualOp::Raw { .. }) => {
                VisualOp::Compose { operations: vec![other, raw] }
            }
        }
    }

    /// Generate WGSL code using the new NormalForm approach
    /// This normalizes the AST first, then generates code with proper warp chaining
    pub fn to_wgsl_normalized(&self) -> String {
        let nf = self.normalize();
        nf.to_wgsl()
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
                arc_to,
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
                    && arc_to.is_none()
                    && alpha_set.is_none()
                    && alpha_mul.is_none()
                    && *length != Rational64::new(1, 1);

                if is_standalone_lm {
                    // Standalone Lm: modify the runtime seg_length variable
                    lines.push(format!("seg_length = seg_length * {:.6};", rational_to_f32(*length)));
                    return lines.join("\n");
                }

                // Direction with optional Bend or ArcTo (cubic Bézier curve)
                if let Some((dx, dy, dz)) = direction {
                    if let Some((tx, ty, tz, k)) = arc_to {
                        // ArcTo with explicit Direction: set direction then arc
                        lines.push(format!(
                            r#"// ArcTo: curve that ends pointing toward target direction
direction = normalize(vec3<f32>({}, {}, {}));
let target_dir = normalize(vec3<f32>({}, {}, {}));
let arc_k = {:.6};
let L = time * velocity;
// End direction: lerp from current toward target by k
let dT = normalize(mix(direction, target_dir, arc_k));
// Bézier control points for smooth arc
let p0 = vec3<f32>(0.0, 0.0, 0.0);
let p3 = p0 + direction * L;  // End position (straight line distance)
let p1 = p0 + direction * (L / 3.0);  // Start tangent = original direction
let p2 = p3 - dT * (L / 3.0);  // End tangent = new direction
// At t=1, position is p3
let pos = p3;
x += pos.x;
y += pos.y;
z += pos.z;
// Update direction to the new end direction
direction = dT;"#,
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz),
                            rational_to_f32(*tx),
                            rational_to_f32(*ty),
                            rational_to_f32(*tz),
                            rational_to_f32(*k)
                        ));
                    } else if let Some((bx, by, bz, k)) = bend {
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
                } else if let Some((tx, ty, tz, k)) = arc_to {
                    // Standalone ArcTo: space warp transform
                    // Rotates position toward target direction, steering through space
                    lines.push(format!(
                        r#"// ArcTo space warp: rotate position toward target direction
{{
    let pos = vec3<f32>(x, y, z);
    let target_dir = normalize(vec3<f32>({}, {}, {}));
    let arc_k = {:.6};
    let t = time / seg_length;  // progress 0→1

    // Current direction approximated from position
    let pos_len = length(pos);
    if (pos_len > 0.001) {{
        let current_dir = pos / pos_len;
        // Rotation axis: perpendicular to both current and target
        let rot_axis_raw = cross(current_dir, target_dir);
        let rot_axis_len = length(rot_axis_raw);

        if (rot_axis_len > 0.001) {{
            let rot_axis = rot_axis_raw / rot_axis_len;
            // Angle between current and target
            let max_angle = acos(clamp(dot(current_dir, target_dir), -1.0, 1.0));
            // Rotate by k * t * max_angle
            let angle = arc_k * t * max_angle;
            let cos_a = cos(angle);
            let sin_a = sin(angle);
            // Rodrigues' rotation formula
            let rotated = pos * cos_a
                        + cross(rot_axis, pos) * sin_a
                        + rot_axis * dot(rot_axis, pos) * (1.0 - cos_a);
            x = rotated.x;
            y = rotated.y;
            z = rotated.z;
        }}
    }}
}}"#,
                        rational_to_f32(*tx),
                        rational_to_f32(*ty),
                        rational_to_f32(*tz),
                        rational_to_f32(*k)
                    ));
                } else if let Some((bx, by, bz, k)) = bend {
                    // Standalone Bend: space warp transform
                    // Bends the accumulated path around an axis
                    // The angle of rotation is proportional to distance from origin
                    // This curves the path while preserving its shape
                    lines.push(format!(
                        r#"// Bend space warp: curve the path around an axis
{{
    let pos = vec3<f32>(x, y, z);
    let bend_axis = normalize(vec3<f32>({}, {}, {}));
    let bend_k = {:.6};

    // Use distance from origin as the "arc length" parameter
    let dist = length(pos);
    if (dist > 0.001) {{
        // Angle proportional to distance traveled (k=1 → 90° per unit distance)
        let angle = bend_k * dist * 1.5708;
        let cos_a = cos(angle);
        let sin_a = sin(angle);

        // Rodrigues' rotation formula
        let rotated = pos * cos_a
                    + cross(bend_axis, pos) * sin_a
                    + bend_axis * dot(bend_axis, pos) * (1.0 - cos_a);
        x = rotated.x;
        y = rotated.y;
        z = rotated.z;
    }}
}}"#,
                        rational_to_f32(*bx),
                        rational_to_f32(*by),
                        rational_to_f32(*bz),
                        rational_to_f32(*k)
                    ));
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
                    // State persists across segments (x,y,z, direction, velocity, scale, alpha, colors)
                    wgsl.push_str(&format!("    // Segment {}\n    {{\n", i));
                    wgsl.push_str(&op.to_wgsl_segment_runtime(base_start, base_end));
                    wgsl.push_str("\n    }\n");
                }

                // After Seq ends, continue moving in the final direction
                let seq_end = current_base_time; // Total duration of all segments
                wgsl.push_str(&format!(
                    r#"    // Continue moving after Seq ends
    {{
        let seq_end = {:.6} * seg_length;
        if (time >= seq_end) {{
            let extra_time = time - seq_end;
            x += direction.x * extra_time * velocity;
            y += direction.y * extra_time * velocity;
            z += direction.z * extra_time * velocity;
        }}
    }}
"#, seq_end));

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

            VisualOp::Raw { wgsl } => {
                // Raw WGSL passes through directly
                wgsl.clone()
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
            arc_to: None,
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

    /// Create an ArcTo operation
    /// tx, ty, tz: target direction vector (where you want to end up pointing)
    /// k: strength factor (1 = fully commit to new direction, 0.5 = halfway)
    pub fn arc_to(tx: Rational64, ty: Rational64, tz: Rational64, k: Rational64) -> Self {
        let mut op = Self::simple_default();
        if let VisualOp::Simple { arc_to: ref mut a, .. } = op {
            *a = Some((tx, ty, tz, k));
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

            VisualOp::Raw { wgsl } => {
                // Raw WGSL passes through directly
                wgsl.clone()
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
                arc_to,
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

                // Direction with optional Bend or ArcTo (cubic Bézier curve)
                if let Some((dx, dy, dz)) = direction {
                    if let Some((tx, ty, tz, k)) = arc_to {
                        // ArcTo: cubic Bézier that changes end direction toward target
                        lines.push(format!(
                            r#"        // ArcTo: curve that ends pointing toward target direction
        let dir = normalize(vec3<f32>({}, {}, {}));
        let target_dir = normalize(vec3<f32>({}, {}, {}));
        let arc_k = {:.6};
        let L = duration * velocity;
        // End direction: lerp from current toward target by k
        let dT = normalize(mix(dir, target_dir, arc_k));
        // Bézier control points: P1 starts in dir, P2 ends in dT
        let p0 = vec3<f32>(0.0, 0.0, 0.0);
        let p3 = dir * L;  // End position (straight line distance)
        let p1 = p0 + dir * (L / 3.0);  // Start tangent = original direction
        let p2 = p3 - dT * (L / 3.0);  // End tangent = new direction
        // Cubic Bézier: (1-t)³P0 + 3(1-t)²tP1 + 3(1-t)t²P2 + t³P3
        let t = progress;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        let pos = mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3;
        if (time >= seg_start) {{
            x += pos.x;
            y += pos.y;
            z += pos.z;
            // Update direction to the new end direction
            direction = dT;
        }}"#,
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz),
                            rational_to_f32(*tx),
                            rational_to_f32(*ty),
                            rational_to_f32(*tz),
                            rational_to_f32(*k)
                        ));
                    } else if let Some((bx, by, bz, k)) = bend {
                        // Bend is symmetric - exit direction same as entry direction
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
        if (time >= seg_start) {{
            x += pos.x;
            y += pos.y;
            z += pos.z;
            direction = dir;
        }}"#,
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
                        // Update global direction so Seq continuation knows which way to go
                        lines.push(format!(
                            "        let dir = normalize(vec3<f32>({}, {}, {}));\n        if (time >= seg_start) {{\n            x += dir.x * dt * velocity;\n            y += dir.y * dt * velocity;\n            z += dir.z * dt * velocity;\n            direction = dir;\n        }}",
                            rational_to_f32(*dx),
                            rational_to_f32(*dy),
                            rational_to_f32(*dz)
                        ));
                    }
                } else if let Some((tx, ty, tz, k)) = arc_to {
                    // Standalone ArcTo: use existing runtime direction
                    lines.push(format!(
                        r#"        // ArcTo: curve using current direction, ending toward target
        let target_dir = normalize(vec3<f32>({}, {}, {}));
        let arc_k = {:.6};
        let L = duration * velocity;
        let dT = normalize(mix(direction, target_dir, arc_k));
        let p0 = vec3<f32>(0.0, 0.0, 0.0);
        let p3 = p0 + direction * L;
        let p1 = p0 + direction * (L / 3.0);
        let p2 = p3 - dT * (L / 3.0);
        let t = progress;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        let pos = mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3;
        if (time >= seg_start) {{
            x += pos.x;
            y += pos.y;
            z += pos.z;
            direction = dT;
        }}"#,
                        rational_to_f32(*tx),
                        rational_to_f32(*ty),
                        rational_to_f32(*tz),
                        rational_to_f32(*k)
                    ));
                } else if let Some((bx, by, bz, k)) = bend {
                    // Standalone Bend: use existing runtime direction
                    lines.push(format!(
                        r#"        // Bézier curve with bend using current direction (symmetric bulge)
        let bend_raw = vec3<f32>({}, {}, {});
        let k = {:.6};
        let L = duration * velocity;
        let bend_perp = bend_raw - dot(bend_raw, direction) * direction;
        let perp_len = length(bend_perp);
        var offset = vec3<f32>(0.0, 0.0, 0.0);
        if (perp_len > 0.001) {{
            let b_perp = bend_perp / perp_len;
            offset = b_perp * k * L * 0.5;
        }}
        let p0 = vec3<f32>(0.0, 0.0, 0.0);
        let p3 = direction * L;
        let p1 = direction * (L / 3.0) + offset;
        let p2 = direction * (L * 2.0 / 3.0) + offset;
        let t = progress;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        let pos = mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3;
        if (time >= seg_start) {{
            x += pos.x;
            y += pos.y;
            z += pos.z;
        }}"#,
                        rational_to_f32(*bx),
                        rational_to_f32(*by),
                        rational_to_f32(*bz),
                        rational_to_f32(*k)
                    ));
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
                // Each inner segment needs its own scope to avoid variable redefinitions
                // State persists across segments (no save/restore)
                let mut wgsl = String::new();
                let mut inner_base_time = base_start;
                let total_len = rational_to_f64(self.length());
                let outer_base_duration = base_end - base_start;

                for (i, op) in items.iter().enumerate() {
                    let inner_duration = rational_to_f64(op.length());
                    let scaled = if total_len > 0.0 {
                        (inner_duration / total_len) * outer_base_duration
                    } else {
                        0.0
                    };
                    // Wrap each inner segment in its own scope (for variable scoping)
                    // State persists across segments
                    wgsl.push_str(&format!("        // Inner segment {}\n        {{\n", i));
                    wgsl.push_str(&op.to_wgsl_segment_runtime(inner_base_time, inner_base_time + scaled));
                    wgsl.push_str("\n        }\n");
                    inner_base_time += scaled;
                }
                wgsl
            }

            VisualOp::Compose { operations } => {
                // Generate code for ALL operations in the Compose
                // Each operation is wrapped in a block scope to allow re-declaration of timing vars
                let mut result = String::new();
                for (i, op) in operations.iter().enumerate() {
                    result.push_str(&format!("        // Compose op {}\n        {{\n", i));
                    // Indent the operation's code
                    let op_code = op.to_wgsl_segment_runtime(base_start, base_end);
                    for line in op_code.lines() {
                        result.push_str("    ");
                        result.push_str(line);
                        result.push('\n');
                    }
                    result.push_str("        }\n");
                }
                result
            }

            VisualOp::Raw { wgsl } => {
                // Raw WGSL needs time-gated execution in a Seq context
                // Generate segment timing and wrap in time check
                // IMPORTANT: Raw WGSL only runs DURING its segment (not after)
                // This differs from position ops which accumulate over time
                let base_duration = base_end - base_start;
                let trimmed = wgsl.trim();
                let with_semi = if trimmed.ends_with(';') || trimmed.ends_with('}') {
                    trimmed.to_string()
                } else {
                    format!("{};", trimmed)
                };
                format!(
                    r#"        let seg_start = {:.6} * seg_length;
        let seg_end = {:.6} * seg_length;
        let duration = {:.6} * seg_length;
        var dt: f32 = 0.0;
        if (time >= seg_end) {{
            dt = duration;
        }} else if (time >= seg_start) {{
            dt = time - seg_start;
        }}
        let progress = select(0.0, dt / duration, duration > 0.0);
        if (time >= seg_start && time < seg_end) {{
            {}
        }}"#,
                    base_start, base_end, base_duration, with_semi
                )
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
fn mix(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> { return a * (1.0 - t) + b * t; }
fn cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> { return vec3<f32>(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x); }
fn dot(a: vec3<f32>, b: vec3<f32>) -> f32 { return a.x * b.x + a.y * b.y + a.z * b.z; }
fn acos(v: f32) -> f32 { return 1.0; }
fn clamp(v: f32, lo: f32, hi: f32) -> f32 { return v; }

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
/// * `block_start_line` - The line number where the WGSL block content starts in the original source
/// * `original_source` - The original full source (for extracting context)
pub fn validate_wgsl_with_position(
    src: &str,
    block_start_line: usize,
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

            // The error_line is in the compiled/patched WGSL (which may differ from source
            // due to DSL expansion). We can't map back to exact DSL positions easily,
            // so we report the error at the WGSL block start and show the compiled line.

            // Extract the error line from compiled code for context
            let user_code_line = if error_line > preamble_lines {
                error_line - preamble_lines
            } else {
                1
            };

            // Get the actual line from compiled src that has the error
            let compiled_context = src
                .lines()
                .nth(user_code_line.saturating_sub(1))
                .unwrap_or("")
                .trim()
                .to_string();

            // Use the block start line for reporting (this is accurate in original source)
            // Add 1 to point to first line inside the block
            let report_line = block_start_line + 1;

            // Get original source context at the block start
            let original_context = original_source
                .lines()
                .nth(report_line)
                .unwrap_or("")
                .to_string();

            // Combine error info: show both the error message and where it occurred in compiled code
            let context = if !compiled_context.is_empty() && compiled_context != original_context.trim() {
                format!("{}\n    (in compiled WGSL: {})", original_context, compiled_context)
            } else {
                original_context
            };

            Err(WgslError {
                message: e.message().to_string(),
                line: report_line,
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

    #[test]
    fn test_visual_op_arc_to() {
        // Test ArcTo creates curve code that changes end direction
        let dir = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        );
        let arc = VisualOp::arc_to(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 1), // k = 1.0 (fully commit to new direction)
        );
        let composed = dir.compose(arc);
        let wgsl = composed.to_wgsl(0.0);

        println!("ArcTo WGSL:\n{}", wgsl);

        // Should have ArcTo components
        assert!(wgsl.contains("ArcTo"), "Output should mention ArcTo");
        assert!(wgsl.contains("target_dir"), "Output should have target direction");
        assert!(wgsl.contains("mix"), "Output should use mix for direction lerp");
        assert!(wgsl.contains("dT"), "Output should have end direction dT");
        assert!(wgsl.contains("direction = dT"), "Should update direction to new end direction");
    }

    #[test]
    fn test_visual_op_arc_to_in_seq() {
        // Test ArcTo works inside Seq with proper Bézier evaluation
        let segment = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::arc_to(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 2), // k = 0.5
        )).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seq = VisualOp::Seq { items: vec![segment] };
        let wgsl = seq.to_wgsl(0.0);

        println!("ArcTo in Seq WGSL:\n{}", wgsl);

        // Should have ArcTo curve with Bézier components
        assert!(wgsl.contains("ArcTo"), "Output should mention ArcTo");
        assert!(wgsl.contains("target_dir"), "Output should have target direction");
        assert!(wgsl.contains("mt3"), "Output should have (1-t)^3 component");
        assert!(wgsl.contains("t3"), "Output should have t^3 component");
    }

    #[test]
    fn test_arc_to_validates() {
        // Ensure generated WGSL code compiles
        let dir = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        );
        let arc = VisualOp::arc_to(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 1),
        );
        let composed = dir.compose(arc);
        let wgsl = composed.to_wgsl(0.0);

        assert!(validate_wgsl(&wgsl).is_ok(), "ArcTo WGSL should validate");
    }

    #[test]
    fn test_standalone_arcto() {
        // Standalone ArcTo should generate space warp transform code
        let arc = VisualOp::arc_to(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 1),
        );
        let wgsl = arc.to_wgsl(0.0);

        println!("Standalone ArcTo WGSL:\n{}", wgsl);

        // Should be a space warp transform
        assert!(wgsl.contains("ArcTo space warp"), "Should be space warp mode");
        assert!(wgsl.contains("let target_dir = normalize"), "Should have target direction");
        assert!(wgsl.contains("Rodrigues"), "Should use Rodrigues rotation");
        assert!(wgsl.contains("cross(rot_axis, pos)"), "Should rotate position");

        let validation_result = validate_wgsl(&wgsl);
        if let Err(ref e) = validation_result {
            eprintln!("Validation error: {}", e);
        }
        assert!(validation_result.is_ok(), "Standalone ArcTo should validate");
    }

    #[test]
    fn test_standalone_bend() {
        // Standalone Bend should generate space warp transform code
        let bend = VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 2),
        );
        let wgsl = bend.to_wgsl(0.0);

        println!("Standalone Bend WGSL:\n{}", wgsl);

        // Should be a space warp transform
        assert!(wgsl.contains("Bend space warp"), "Should be space warp mode");
        assert!(wgsl.contains("bend_axis = normalize"), "Should have bend axis");
        assert!(wgsl.contains("Rodrigues"), "Should use Rodrigues rotation");

        assert!(validate_wgsl(&wgsl).is_ok(), "Standalone Bend should validate");
    }

    #[test]
    fn test_seq_with_bend_warp_normalized() {
        // Test Seq | Bend using the new NormalForm approach
        // The Bend should apply as a warp to the direction in each segment

        // Create a simple Seq with two directions
        let seg1 = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seg2 = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(1, 1)));

        let seq = VisualOp::Seq { items: vec![seg1, seg2] };

        // Compose with a Bend warp
        let bend = VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 2), // k = 0.5
        );

        let composed = seq.compose(bend);

        // Normalize and generate WGSL
        let nf = composed.normalize();
        println!("NormalForm: {} operations, length={:?}", nf.operations.len(), nf.length);

        // Each segment should have the Bend warp
        assert_eq!(nf.operations.len(), 2, "Should have 2 segments");
        assert_eq!(nf.operations[0].warps.len(), 1, "First segment should have 1 warp");
        assert_eq!(nf.operations[1].warps.len(), 1, "Second segment should have 1 warp");

        // Generate WGSL
        let wgsl = nf.to_wgsl();
        println!("Normalized WGSL:\n{}", wgsl);

        // Should contain Bend warp code
        assert!(wgsl.contains("Bend warp"), "Should have Bend warp code");
        assert!(wgsl.contains("Rodrigues"), "Should use Rodrigues rotation");
        assert!(wgsl.contains("direction ="), "Should modify direction");

        // Validate the WGSL
        let validation_result = validate_wgsl(&wgsl);
        if let Err(ref e) = validation_result {
            eprintln!("Validation error: {}", e);
        }
        assert!(validation_result.is_ok(), "Normalized Seq|Bend should validate");
    }

    #[test]
    fn test_chained_warps_normalized() {
        // Test chaining multiple warps: Seq | Bend | ArcTo
        let seg = VisualOp::direction(
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(0, 1),
        ).compose(VisualOp::lm(Rational64::new(2, 1)));

        let seq = VisualOp::Seq { items: vec![seg] };

        // Chain two warps
        let bend = VisualOp::bend(
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 2),
        );
        let arc_to = VisualOp::arc_to(
            Rational64::new(0, 1),
            Rational64::new(0, 1),
            Rational64::new(1, 1),
            Rational64::new(1, 4),
        );

        let composed = seq.compose(bend).compose(arc_to);
        let nf = composed.normalize();

        println!("Chained warps NormalForm: {} operations", nf.operations.len());
        for (i, op) in nf.operations.iter().enumerate() {
            println!("  Segment {}: {} warps", i, op.warps.len());
        }

        // Should have both warps chained
        assert_eq!(nf.operations.len(), 1, "Should have 1 segment");
        assert_eq!(nf.operations[0].warps.len(), 2, "Segment should have 2 chained warps");

        // Generate and validate WGSL
        let wgsl = nf.to_wgsl();
        println!("Chained warps WGSL:\n{}", wgsl);

        assert!(wgsl.contains("Bend warp"), "Should have Bend warp code");
        assert!(wgsl.contains("ArcTo warp"), "Should have ArcTo warp code");

        let validation_result = validate_wgsl(&wgsl);
        if let Err(ref e) = validation_result {
            eprintln!("Validation error: {}", e);
        }
        assert!(validation_result.is_ok(), "Chained warps should validate");
    }

    #[test]
    fn test_seq_compose_seq() {
        // Test Seq | Seq behavior
        // Inner Seq plays twice with different modifiers
        let inner_seq = VisualOp::Seq {
            items: vec![
                VisualOp::direction(
                    Rational64::new(1, 1),
                    Rational64::new(0, 1),
                    Rational64::new(0, 1),
                ).compose(VisualOp::lm(Rational64::new(1, 1))),
            ],
        };

        let outer_seq = VisualOp::Seq {
            items: vec![
                VisualOp::vm(Rational64::new(1, 1)),
                VisualOp::vm(Rational64::new(2, 1)),
            ],
        };

        let composed = inner_seq.compose(outer_seq);

        println!("=== Composed AST ===");
        println!("{:#?}", composed);
        println!("\n=== Generated WGSL ===");
        let wgsl = composed.to_wgsl(0.0);
        println!("{}", wgsl);

        // The Seq should appear twice with different time ranges
        // First iteration: time 0-1 with Vm 1
        // Second iteration: time 1-2 with Vm 2
        assert!(wgsl.contains("Segment 0"), "Should have segment 0");
        assert!(wgsl.contains("Segment 1"), "Should have segment 1");
    }
}

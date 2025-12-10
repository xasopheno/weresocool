//! Core formatting logic using pretty.rs

/// Indentation size (number of spaces per level)
const INDENT: isize = 4;

use crate::FormatConfig;
use num_rational::Rational64;
use pretty::{Arena, DocAllocator, DocBuilder};
use weresocool_ast::{
    FunDef, ListOp, GenOp, Op, Term,
    operations::Defs,
};
use weresocool_parser::{Init, ParsedComposition};

/// Context for formatting that carries defs for lookups (colors, wgsl, etc.)
struct FormatContext<'a> {
    defs: &'a Defs,
    /// Original source text (for final output)
    original_source: Option<&'a str>,
    /// Span source (processed, for span lookups - may differ from original if WGSL was processed)
    span_source: Option<&'a str>,
}

/// Format a complete parsed composition
pub fn format_composition(parsed: &ParsedComposition, config: &FormatConfig) -> String {
    format_composition_with_source(parsed, "", "", config)
}

/// Format with access to original source for better preservation
/// Comments are preserved through normalize_def_whitespace - no separate extraction needed
pub fn format_composition_with_source(parsed: &ParsedComposition, original_source: &str, span_source: &str, config: &FormatConfig) -> String {
    let arena = Arena::new();
    let doc = format_composition_doc(&arena, parsed, original_source, span_source, config);
    let mut output = String::new();
    doc.render_fmt(config.max_width, &mut output).unwrap();
    output
}

/// Format just an init block
pub fn format_init_block(init: &Init, config: &FormatConfig) -> String {
    let arena = Arena::new();
    let doc = format_init(&arena, init);
    let mut output = String::new();
    doc.render_fmt(config.max_width, &mut output).unwrap();
    output
}

fn format_composition_doc<'a>(
    arena: &'a Arena<'a>,
    parsed: &ParsedComposition,
    original_source: &'a str,
    span_source: &'a str,
    _config: &FormatConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let ctx = FormatContext {
        defs: &parsed.defs,
        original_source: if original_source.is_empty() { None } else { Some(original_source) },
        span_source: if span_source.is_empty() { None } else { Some(span_source) },
    };
    let init_doc = format_init(arena, &parsed.init);

    // Get definitions from global scope
    let mut defs_docs: Vec<DocBuilder<'a, Arena<'a>>> = Vec::new();
    for (scope_name, scope) in parsed.defs.ops.iter() {
        // Only format global scope definitions
        if scope_name == "global" {
            for (name, term) in scope.iter() {
                // Try to use span-based source preservation first
                // Use span_source for lookups (it matches the spans), but original_source for content
                if let (Some(orig_src), Some(span_src)) = (ctx.original_source, ctx.span_source) {
                    if let Some(span) = parsed.defs.spans.get_def(name) {
                        if span.end <= span_src.len() {
                            // Find corresponding position in original source
                            // For now, use original source directly since we need the WGSL content
                            // This works because definition names are the same in both sources
                            if let Some(orig_span) = find_def_in_source(orig_src, name) {
                                let def_text = &orig_src[orig_span.0..orig_span.1];
                                let normalized = normalize_def_whitespace(def_text);
                                defs_docs.push(arena.text(normalized));
                                continue;
                            }
                        }
                    }
                }

                // Fallback to AST-based formatting
                let def_doc = match term {
                    Term::FunDef(fun) => format_fundef(arena, &ctx, fun),
                    _ => {
                        let term_doc = format_term(arena, &ctx, term);
                        arena
                            .text(name.clone())
                            .append(arena.text(" = "))
                            .append(term_doc)
                    }
                };
                defs_docs.push(def_doc);
            }
        }
    }

    // Build final document: init block, then definitions separated by blank lines
    let mut result = init_doc;
    for def in defs_docs {
        result = result
            .append(arena.hardline())
            .append(arena.hardline())
            .append(def);
    }
    result.append(arena.hardline())
}

/// Normalize whitespace in a definition while preserving tokens
fn normalize_def_whitespace(def: &str) -> String {
    let mut result = String::new();
    let mut chars = def.chars().peekable();
    let mut indent_level: usize = 0;
    let mut at_line_start = true;
    let mut in_string = false;
    let mut in_comment = false;
    let mut prev_was_space = false;

    while let Some(ch) = chars.next() {
        // Handle comments: -- or // style
        if !in_string && !in_comment && ch == '-' && chars.peek() == Some(&'-') {
            // Start of -- comment
            if at_line_start {
                add_indent(&mut result, indent_level);
                at_line_start = false;
            }
            result.push(ch);
            result.push(chars.next().unwrap()); // consume second -
            in_comment = true;
            prev_was_space = false;
            continue;
        }
        if !in_string && !in_comment && ch == '/' && chars.peek() == Some(&'/') {
            // Start of // comment
            if at_line_start {
                add_indent(&mut result, indent_level);
                at_line_start = false;
            }
            result.push(ch);
            result.push(chars.next().unwrap()); // consume second /
            in_comment = true;
            prev_was_space = false;
            continue;
        }

        // Inside a comment - pass through everything until newline
        if in_comment {
            if ch == '\n' {
                in_comment = false;
                // Remove trailing whitespace from comment
                while result.ends_with(' ') {
                    result.pop();
                }
                result.push('\n');
                at_line_start = true;
                prev_was_space = false;
            } else {
                result.push(ch);
            }
            continue;
        }

        // Track string state
        if ch == '"' && !in_string {
            in_string = true;
            if at_line_start {
                add_indent(&mut result, indent_level);
                at_line_start = false;
            }
            result.push(ch);
            prev_was_space = false;
            continue;
        } else if ch == '"' && in_string {
            in_string = false;
            result.push(ch);
            prev_was_space = false;
            continue;
        }

        if in_string {
            result.push(ch);
            continue;
        }

        match ch {
            '{' => {
                if at_line_start {
                    add_indent(&mut result, indent_level);
                }
                result.push(ch);
                indent_level += 1;
                at_line_start = false;
                prev_was_space = false;

                // Peek ahead to see if content follows on same line
                skip_whitespace_except_newline(&mut chars);
                if chars.peek() == Some(&'\n') {
                    // Content on next line - consume the newline
                    chars.next();
                    result.push('\n');
                    at_line_start = true;
                } else if chars.peek().is_some() {
                    result.push(' ');
                }
            }
            '}' => {
                indent_level = indent_level.saturating_sub(1);
                if at_line_start {
                    add_indent(&mut result, indent_level);
                } else if !prev_was_space && !result.ends_with('\n') {
                    result.push(' ');
                }
                result.push(ch);
                at_line_start = false;
                prev_was_space = false;
            }
            '[' => {
                if at_line_start {
                    add_indent(&mut result, indent_level);
                }
                result.push(ch);
                indent_level += 1;
                at_line_start = false;
                prev_was_space = false;

                // Peek ahead for newline
                skip_whitespace_except_newline(&mut chars);
                if chars.peek() == Some(&'\n') {
                    // Look ahead to see if the ] is on the next non-blank line with content
                    // If so, keep it on one line (e.g., "Overlay [\n  Fm 1, Fm 2]" -> "Overlay [Fm 1, Fm 2]")
                    let remaining: String = chars.clone().collect();
                    let should_collapse = should_collapse_bracket(&remaining);

                    if should_collapse {
                        // Skip the newline and whitespace, content will follow
                        chars.next(); // consume newline
                        while chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                        skip_whitespace_except_newline(&mut chars);
                        // Don't add newline - keep on same line
                    } else {
                        chars.next(); // consume the newline
                        // Skip any additional blank lines
                        while chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                        // Skip any leading whitespace on the next content line
                        skip_whitespace_except_newline(&mut chars);
                        result.push('\n');
                        at_line_start = true;
                    }
                }
            }
            ']' => {
                indent_level = indent_level.saturating_sub(1);
                if at_line_start {
                    add_indent(&mut result, indent_level);
                }
                result.push(ch);
                at_line_start = false;
                prev_was_space = false;
            }
            '\n' => {
                // Remove trailing whitespace
                while result.ends_with(' ') {
                    result.pop();
                }
                // Skip blank lines entirely inside brackets/braces (indent_level > 0)
                // This handles cases where comments were stripped leaving blank lines
                if indent_level > 0 && result.ends_with('\n') {
                    // Inside a block and previous line was also a newline - skip
                } else {
                    result.push('\n');
                }
                at_line_start = true;
                prev_was_space = false;
            }
            ' ' | '\t' => {
                if at_line_start {
                    // Skip leading whitespace - we add our own indent
                } else if !prev_was_space && !result.ends_with('\n') {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            '|' => {
                if at_line_start {
                    add_indent(&mut result, indent_level);
                } else if !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
                result.push(ch);
                at_line_start = false;
                prev_was_space = false;
                // Add space after pipe
                skip_whitespace_except_newline(&mut chars);
                if chars.peek() != Some(&'\n') && chars.peek().is_some() {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            ',' => {
                result.push(ch);
                at_line_start = false;
                prev_was_space = false;
                // Add space after comma if not followed by newline
                skip_whitespace_except_newline(&mut chars);
                if chars.peek() != Some(&'\n') && chars.peek().is_some() {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            _ => {
                if at_line_start {
                    add_indent(&mut result, indent_level);
                    at_line_start = false;
                }
                result.push(ch);
                prev_was_space = false;
            }
        }
    }

    // Remove trailing whitespace
    while result.ends_with(' ') || result.ends_with('\n') {
        result.pop();
    }

    result
}

fn add_indent(result: &mut String, level: usize) {
    for _ in 0..(level * INDENT as usize) {
        result.push(' ');
    }
}

fn skip_whitespace_except_newline(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&ch) = chars.peek() {
        if ch == ' ' || ch == '\t' {
            chars.next();
        } else {
            break;
        }
    }
}

/// Check if content after '[' should be collapsed onto the same line.
/// Pattern: "[\n  content]" where ] is on same line as content -> "[content]"
/// We want to collapse when there's one newline followed by content ending with ]
fn should_collapse_bracket(remaining: &str) -> bool {
    // remaining starts after '[', should be "\n  content...]..."

    // Skip the initial newline and whitespace
    let after_newline = remaining.trim_start_matches(|c| c == '\n' || c == ' ' || c == '\t');

    // Find where this line ends (next newline or end)
    let line_end = after_newline.find('\n').unwrap_or(after_newline.len());
    let first_line = &after_newline[..line_end];

    // Check if this line contains the closing ] at depth 0
    let mut depth = 0;
    for ch in first_line.chars() {
        match ch {
            '[' | '{' => depth += 1,
            ']' => {
                if depth == 0 {
                    // Found closing ] on this line - should collapse
                    return true;
                }
                depth -= 1;
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }

    false
}

/// Find a definition by name in the source, returning (start, end) positions
fn find_def_in_source(source: &str, name: &str) -> Option<(usize, usize)> {
    // Look for patterns like "name = {" or "name(args) = {"
    let pattern_simple = format!("{} = {{", name);
    let pattern_func = format!("{}(", name);

    // Find the start of the definition
    let start = if let Some(pos) = source.find(&pattern_simple) {
        pos
    } else if let Some(pos) = source.find(&pattern_func) {
        pos
    } else {
        return None;
    };

    // Find the opening brace after the name
    let after_name = &source[start..];
    let brace_offset = after_name.find('{')?;
    let brace_pos = start + brace_offset;

    // Find matching closing brace
    let mut depth = 1;
    let mut pos = brace_pos + 1;
    let bytes = source.as_bytes();
    let mut in_string = false;

    while pos < bytes.len() && depth > 0 {
        let ch = bytes[pos];

        // Handle strings (simplified)
        if ch == b'"' {
            in_string = !in_string;
        }

        if !in_string {
            if ch == b'{' {
                depth += 1;
            } else if ch == b'}' {
                depth -= 1;
            }
        }

        pos += 1;
    }

    if depth == 0 {
        Some((start, pos))
    } else {
        None
    }
}

fn format_init<'a>(arena: &'a Arena<'a>, init: &Init) -> DocBuilder<'a, Arena<'a>> {
    // { f: 311.127, l: 1, g: 1/1, p: 0 }
    let f_doc = arena
        .text("f: ")
        .append(format_rational(arena, &init.f));
    let l_doc = arena
        .text("l: ")
        .append(format_rational(arena, &init.l));
    let g_doc = arena
        .text("g: ")
        .append(format_rational(arena, &init.g));
    let p_doc = arena
        .text("p: ")
        .append(format_rational(arena, &init.p));

    let fields = arena.intersperse(
        [f_doc, l_doc, g_doc, p_doc],
        arena.text(",").append(arena.space()),
    );

    arena
        .text("{ ")
        .append(fields)
        .append(arena.text(" }"))
}

fn format_rational<'a>(arena: &'a Arena<'a>, r: &Rational64) -> DocBuilder<'a, Arena<'a>> {
    let denom = *r.denom();
    let numer = *r.numer();

    if denom == 1 {
        // Integer
        arena.text(numer.to_string())
    } else if looks_like_float(numer, denom) {
        // Looks like it was originally a float - format as decimal
        let decimal_places = (denom as f64).log10() as usize;
        let value = numer as f64 / denom as f64;
        arena.text(format!("{:.prec$}", value, prec = decimal_places))
    } else {
        // Fraction
        arena.text(format!("{}/{}", numer, denom))
    }
}

/// Heuristic to detect if a rational was originally written as a float.
/// A float-originated rational has:
/// - Denominator that is a power of 10
/// - Numerator that is NOT evenly divisible by a simple factor that would make it "pretty"
///
/// For example:
/// - 91/10 (from 9.1) → true (91 doesn't reduce to a nice fraction with denom 10)
/// - 9/10 (from writing 9/10) → false (this looks like an intentional fraction)
/// - 5/10 → false (reduces to 1/2, so user would have written 1/2)
fn looks_like_float(numer: i64, denom: i64) -> bool {
    if !is_power_of_10(denom) {
        return false;
    }

    // If the fraction would reduce to a simpler form, it's probably intentional
    // Check if gcd(numer, denom) > 1 - if so, the fraction would have been reduced
    // Since Rational64 auto-reduces, if we're here with denom as power of 10,
    // it means gcd(numer, denom) == 1

    // Special case: very small denominators like 10 could be intentional fractions
    // like 9/10, 7/10, 3/10. These are common musical ratios.
    // But 91/10, 127/10 are almost certainly from floats.

    // Heuristic: if |numer| > denom, it was probably a float
    // Because 9/10, 3/10, 1/10 are common fractions, but 91/10, 311/10 are floats
    numer.abs() > denom
}

/// Check if n is a power of 10
fn is_power_of_10(n: i64) -> bool {
    if n <= 0 {
        return false;
    }
    let mut x = n;
    while x > 1 {
        if x % 10 != 0 {
            return false;
        }
        x /= 10;
    }
    x == 1
}

fn format_term<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, term: &Term) -> DocBuilder<'a, Arena<'a>> {
    match term {
        Term::Op(op) => format_op(arena, ctx, op),
        Term::Nf(_) => arena.text("<NormalForm>"), // NormalForms shouldn't appear in source
        Term::FunDef(fun) => format_fundef(arena, ctx, fun),
        Term::Lop(lop) => format_listop(arena, ctx, lop),
        Term::Gen(gen) => format_genop(arena, ctx, gen),
    }
}

fn format_fundef<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, fun: &FunDef) -> DocBuilder<'a, Arena<'a>> {
    // name(arg1, arg2) = { ... }
    let args = arena.intersperse(
        fun.vars.iter().map(|v| arena.text(v.clone())),
        arena.text(", "),
    );
    let body = format_term(arena, ctx, &fun.term);

    // Check if body is simple (single op) or complex
    let is_simple = matches!(&*fun.term, Term::Op(Op::Id(_)) | Term::Op(Op::AsIs));

    if is_simple {
        // Simple body: f(a) = { a }
        arena
            .text(fun.name.clone())
            .append(arena.text("("))
            .append(args)
            .append(arena.text(") = { "))
            .append(body)
            .append(arena.text(" }"))
    } else {
        // Complex body: wrap in braces with proper nesting
        arena
            .text(fun.name.clone())
            .append(arena.text("("))
            .append(args)
            .append(arena.text(") = {"))
            .append(arena.line().append(body).nest(2).group())
            .append(arena.line())
            .append(arena.text("}"))
            .group()
    }
}

fn format_op<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, op: &Op) -> DocBuilder<'a, Arena<'a>> {
    match op {
        // Simple ops
        Op::AsIs => arena.text("AsIs"),
        Op::Out => arena.text("Out"),
        Op::Reverse => arena.text("Reverse"),
        Op::FInvert => arena.text("FInvert"),
        Op::Noise => arena.text("Noise"),
        Op::Saw => arena.text("Saw"),

        // Identifiers
        Op::Id(name) => arena.text(name.clone()),
        Op::Tag(name) => arena.text(format!("@{}", name)),
        Op::Keeper(name) => arena.text(format!("${}", name)),

        // Sine, Triangle, Square with optional parameters
        Op::Sine { pow } => format_osc_op(arena, "Sine", pow),
        Op::Triangle { pow } => format_osc_op(arena, "Triangle", pow),
        Op::Square { width } => format_osc_op(arena, "Square", width),

        // Single rational parameter ops
        // NOTE: Syntax preservation will be implemented in Phase 4 using FormatTree
        Op::TransposeM { m } => format_single_rational_op(arena, "Fm", m),
        Op::TransposeA { a } => format_single_rational_op(arena, "Fa", a),
        Op::PanM { m } => format_single_rational_op(arena, "PanM", m),
        Op::PanA { a } => format_single_rational_op(arena, "PanA", a),
        Op::Gain { m } => format_single_rational_op(arena, "Gain", m),
        Op::Length { m } => format_single_rational_op(arena, "Lm", m),
        Op::Silence { m } => format_single_rational_op(arena, "Silence", m),
        Op::Portamento { m } => format_single_rational_op(arena, "Portamento", m),

        Op::Reverb { m } => {
            if let Some(val) = m {
                format_single_rational_op(arena, "Reverb", val)
            } else {
                arena.text("Reverb")
            }
        }

        // AD envelope
        Op::AD { attack, decay, asr } => {
            let asr_str = match asr {
                weresocool_ast::ASR::Short => "Short",
                weresocool_ast::ASR::Long => "Long",
            };
            arena
                .text("AD ")
                .append(format_rational(arena, attack))
                .append(arena.text(", "))
                .append(format_rational(arena, decay))
                .append(arena.text(", "))
                .append(arena.text(asr_str))
        }

        // Collections
        // NOTE: Syntax preservation will be implemented in Phase 4 using FormatTree
        Op::Sequence { operations } => {
            // Check if this is a Repeat (all AsIs)
            let all_asis = operations.iter().all(|op| matches!(op, Term::Op(Op::AsIs)));
            if all_asis && !operations.is_empty() {
                arena
                    .text("Repeat ")
                    .append(arena.text(operations.len().to_string()))
            } else {
                format_collection(arena, ctx, "Seq", operations)
            }
        }
        Op::Overlay { operations } => format_collection(arena, ctx, "Overlay", operations),
        Op::Choose { operations } => format_collection(arena, ctx, "Choose", operations),

        // Compose (pipe chains)
        // NOTE: Syntax preservation will be implemented in Phase 4 using FormatTree
        Op::Compose { operations } => format_compose(arena, ctx, operations),

        // Repeat
        Op::Repeat { operations, count } => {
            // Check if it's a single AsIs (meaning "repeat previous in pipe")
            let is_single_asis = operations.len() == 1
                && matches!(&operations[0], Term::Op(Op::AsIs));

            if is_single_asis {
                // Just output "Repeat N" - used in pipe context
                arena
                    .text("Repeat ")
                    .append(arena.text(count.to_string()))
            } else if operations.len() == 1 {
                // Single operation: format as { op | Repeat N }
                let op_doc = format_term(arena, ctx, &operations[0]);
                arena
                    .text("{")
                    .append(arena.line())
                    .append(op_doc)
                    .append(arena.line())
                    .append(arena.text("| Repeat "))
                    .append(arena.text(count.to_string()))
                    .append(arena.line())
                    .append(arena.text("}"))
                    .nest(2)
                    .group()
            } else {
                // Multiple operations: format as pipe chain ending with Repeat N
                // NO braces here - let the outer Compose handle braces
                let formatted: Vec<_> = operations.iter().map(|t| format_term(arena, ctx, t)).collect();
                let sep = arena.line().append(arena.text("| "));
                let inner = arena.intersperse(formatted, sep);

                // Add Repeat at the end
                inner
                    .append(arena.line())
                    .append(arena.text("| Repeat "))
                    .append(arena.text(count.to_string()))
            }
        }

        // ModulateBy
        Op::ModulateBy { operations, output } => {
            let inner = format_term_list(arena, ctx, operations);
            let base = arena.text("ModBy ").append(inner);
            if let Some(out) = output {
                let out_doc = format_term_list(arena, ctx, out);
                base.append(arena.text(" -> ")).append(out_doc)
            } else {
                base
            }
        }

        // Function call
        Op::FunctionCall { name, args } => {
            let args_doc = arena.intersperse(
                args.iter().map(|t| format_term(arena, ctx, t)),
                arena.text(", "),
            );
            arena
                .text(name.clone())
                .append(arena.text("("))
                .append(args_doc)
                .append(arena.text(")"))
        }

        // Lambda
        Op::Lambda { input_name, term, .. } => {
            let body = format_term(arena, ctx, term);
            if let Some(name) = input_name {
                arena
                    .text("|")
                    .append(arena.text(name.clone()))
                    .append(arena.text("| "))
                    .append(body)
            } else {
                arena.text("|| ").append(body)
            }
        }

        // Filters
        Op::Lowpass { cutoff_frequency, q_factor, .. } => {
            format_filter_op(arena, "Lp", cutoff_frequency, q_factor)
        }
        Op::Highpass { cutoff_frequency, q_factor, .. } => {
            format_filter_op(arena, "Hp", cutoff_frequency, q_factor)
        }
        Op::Bandpass { cutoff_frequency, q_factor, .. } => {
            format_filter_op(arena, "Bp", cutoff_frequency, q_factor)
        }

        // WGSL - lookup the ORIGINAL source from defs (preserves DSL syntax like Ym 2/3)
        Op::WGSL(id) => {
            if let Some(code) = ctx.defs.wgsl.get_original(&(*id as u8)) {
                // Normalize WGSL code: trim each line
                let lines: Vec<String> = code.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();

                if lines.len() == 1 {
                    // Single line - keep compact
                    arena.text(format!("wgsl {{ {} }}", lines[0]))
                } else {
                    // Multiple lines - use pretty.rs for proper nesting
                    let line_docs: Vec<_> = lines.iter()
                        .map(|l| arena.text(l.clone()))
                        .collect();
                    let inner = arena.intersperse(line_docs, arena.hardline());

                    arena.text("wgsl {")
                        .append(arena.hardline())
                        .append(inner)
                        .nest(4)
                        .append(arena.hardline())
                        .append(arena.text("}"))
                }
            } else {
                arena.text(format!("wgsl {{ /* block {} */ }}", id))
            }
        }

        // Color operations - lookup from defs
        Op::Color(id) => {
            if let Some(color) = ctx.defs.colors.get_by_hash(id.to_string()) {
                format_color_value(arena, color)
            } else {
                arena.text(format!("Color {}", id))
            }
        }
        Op::Hue { value } => format_single_rational_op(arena, "Hue", value),
        Op::Saturation { value } => format_single_rational_op(arena, "Saturation", value),
        Op::Brightness { value } => format_single_rational_op(arena, "Brightness", value),
        Op::Vibrance { value } => format_single_rational_op(arena, "Vibrance", value),
        Op::Gamma { value } => format_single_rational_op(arena, "Gamma", value),
        Op::ColorBlend { color_id, amount } => {
            arena
                .text("ColorBlend ")
                .append(arena.text(color_id.to_string()))
                .append(arena.text(", "))
                .append(format_rational(arena, amount))
        }
        Op::ColorAdd { color_id } => {
            arena.text("ColorAdd ").append(arena.text(color_id.to_string()))
        }
        Op::ColorGradient { x, y, z } => {
            arena.text(format!("Gradient({}, {}, {})", x, y, z))
        }
        Op::ColorMix { amount } => {
            format_single_rational_op(arena, "Mix", amount)
        }

        // MIDI
        Op::Midi { channels } => {
            let chs = channels.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ");
            arena.text(format!("Midi [{}]", chs))
        }

        // FM Oscillator
        Op::FMOsc { defs } => {
            let defs_str = defs
                .iter()
                .map(|d| format!("({}, {})", d.fm, d.depth))
                .collect::<Vec<_>>()
                .join(", ");
            arena.text(format!("FMOsc [{}]", defs_str))
        }

        // Distortion effects - use function call syntax
        Op::Wavefolder { threshold, stages, input_gain, output_gain } => {
            arena.text(format!(
                "Wavefolder({}, {}, {}, {})",
                threshold, stages, input_gain, output_gain
            ))
        }
        Op::SoftClip { threshold, input_gain, output_gain } => {
            arena.text(format!(
                "SoftClip({}, {}, {})",
                threshold, input_gain, output_gain
            ))
        }
        Op::Overdrive { input_gain, output_gain } => {
            arena.text(format!(
                "Overdrive({}, {})",
                input_gain, output_gain
            ))
        }
        Op::Bitcrusher { bits, input_gain, output_gain } => {
            arena.text(format!(
                "Bitcrusher({}, {}, {})",
                bits, input_gain, output_gain
            ))
        }
        Op::Tanh { input_gain, output_gain } => {
            arena.text(format!(
                "Tanh({}, {})",
                input_gain, output_gain
            ))
        }

        // CSV ops
        Op::CSV1d { path, scale } => {
            if let Some(s) = scale {
                arena.text(format!("CSV1d \"{}\" {}", path, s))
            } else {
                arena.text(format!("CSV1d \"{}\"", path))
            }
        }
        Op::CSV2d { path, .. } => arena.text(format!("CSV2d \"{}\"", path)),

        // WithLengthRatioOf
        // Note: When this appears in a Compose, the `main` content is already
        // formatted as a preceding element in the pipe chain (due to how
        // handle_fit_length_recursively works), so we just output "FitLength X"
        Op::WithLengthRatioOf { main: _, with_length_of } => {
            let length_of_doc = format_term(arena, ctx, with_length_of);
            arena.text("FitLength ").append(length_of_doc)
        }

        // Focus
        Op::Focus { name, main, op_to_apply } => {
            let main_doc = format_term(arena, ctx, main);
            let op_doc = format_term(arena, ctx, op_to_apply);
            arena
                .text("Focus ")
                .append(arena.text(name.clone()))
                .append(arena.text(" "))
                .append(main_doc)
                .append(arena.text(" "))
                .append(op_doc)
        }

        // Follow - complex, just placeholder for now
        Op::Follow(_) => arena.text("Follow { ... }"),
    }
}

fn format_single_rational_op<'a>(
    arena: &'a Arena<'a>,
    name: &'static str,
    value: &Rational64,
) -> DocBuilder<'a, Arena<'a>> {
    arena
        .text(name)
        .append(arena.text(" "))
        .append(format_rational(arena, value))
}

fn format_osc_op<'a>(
    arena: &'a Arena<'a>,
    name: &'static str,
    param: &Option<Rational64>,
) -> DocBuilder<'a, Arena<'a>> {
    if let Some(val) = param {
        arena
            .text(name)
            .append(arena.text(" "))
            .append(format_rational(arena, val))
    } else {
        arena.text(name)
    }
}

fn format_filter_op<'a>(
    arena: &'a Arena<'a>,
    name: &'static str,
    cutoff: &Rational64,
    q: &Rational64,
) -> DocBuilder<'a, Arena<'a>> {
    arena
        .text(name)
        .append(arena.text(" "))
        .append(format_rational(arena, cutoff))
        .append(arena.text(", "))
        .append(format_rational(arena, q))
}

fn format_collection<'a>(
    arena: &'a Arena<'a>,
    ctx: &FormatContext,
    name: &'static str,
    items: &[Term],
) -> DocBuilder<'a, Arena<'a>> {
    let inner = format_term_list(arena, ctx, items);

    // Smart grouping: try to fit on one line, otherwise break
    arena
        .text(name)
        .append(arena.text(" "))
        .append(inner)
}

/// Format overtone overlay using the O[...] notation with (f, a, g, p) tuples
fn format_overtone_overlay<'a>(
    arena: &'a Arena<'a>,
    ctx: &FormatContext,
    items: &[Term],
) -> DocBuilder<'a, Arena<'a>> {
    if items.is_empty() {
        return arena.text("O []");
    }

    // Format each item - these should be Compose operations with 4 components
    let formatted_items: Vec<_> = items.iter().map(|t| format_overtone_tuple(arena, ctx, t)).collect();

    // Create comma-separated list with smart line breaking
    let sep = arena.text(",").append(arena.line());
    let inner = arena.intersperse(formatted_items, sep);

    arena
        .text("O [")
        .append(arena.line_().append(inner).nest(2).group())
        .append(arena.line_())
        .append(arena.text("]"))
        .group()
}

/// Format a single overtone tuple (f, a, g, p)
fn format_overtone_tuple<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, term: &Term) -> DocBuilder<'a, Arena<'a>> {
    // An overtone tuple should be a Compose with 4 elements: TransposeM, TransposeA, Gain, PanA
    if let Term::Op(Op::Compose { operations, .. }) = term {
        if operations.len() == 4 {
            let mut f = None;
            let mut a = None;
            let mut g = None;
            let mut p = None;

            for op in operations {
                match op {
                    Term::Op(Op::TransposeM { m, .. }) => f = Some(*m),
                    Term::Op(Op::TransposeA { a: val, .. }) => a = Some(*val),
                    Term::Op(Op::Gain { m, .. }) => g = Some(*m),
                    Term::Op(Op::PanA { a: val, .. }) => p = Some(*val),
                    _ => {}
                }
            }

            if let (Some(f), Some(a), Some(g), Some(p)) = (f, a, g, p) {
                return arena
                    .text("(")
                    .append(format_rational(arena, &f))
                    .append(arena.text(", "))
                    .append(format_rational(arena, &a))
                    .append(arena.text(", "))
                    .append(format_rational(arena, &g))
                    .append(arena.text(", "))
                    .append(format_rational(arena, &p))
                    .append(arena.text(")"));
            }
        }
    }

    // Fallback to normal term formatting if not a proper overtone tuple
    format_term(arena, ctx, term)
}

fn format_term_list<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, items: &[Term]) -> DocBuilder<'a, Arena<'a>> {
    if items.is_empty() {
        return arena.text("[]");
    }

    // Format items for use inside collection brackets - no extra braces around pipe chains
    let formatted_items: Vec<_> = items.iter().map(|t| format_term_in_collection(arena, ctx, t)).collect();

    // Check if any item is complex (contains pipe chains, nested collections, etc.)
    let has_complex_item = items.iter().any(|t| is_complex_term(t));

    // For 3+ items with complex content, force multiple lines
    if items.len() >= 3 && has_complex_item {
        let sep = arena.text(",").append(arena.hardline());
        let inner = arena.intersperse(formatted_items, sep);

        arena
            .text("[")
            .append(arena.hardline().append(inner).nest(2))
            .append(arena.hardline())
            .append(arena.text("]"))
    } else {
        // Use smart line breaking - will break if doesn't fit
        let sep = arena.text(",").append(arena.line());
        let inner = arena.intersperse(formatted_items, sep);

        arena
            .text("[")
            .append(arena.line_().append(inner).nest(2).group())
            .append(arena.line_())
            .append(arena.text("]"))
            .group()
    }
}

/// Format a term that appears inside a collection (Overlay, Seq, Choose brackets).
/// Pipe chains should NOT be wrapped in braces here - the brackets provide the grouping.
fn format_term_in_collection<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, term: &Term) -> DocBuilder<'a, Arena<'a>> {
    match term {
        Term::Op(Op::Compose { operations }) => {
            // Format compose WITHOUT braces when inside a collection
            format_compose_inner(arena, ctx, operations)
        }
        _ => format_term(arena, ctx, term),
    }
}

/// Check if a term is complex enough to warrant line breaking
fn is_complex_term(term: &Term) -> bool {
    match term {
        Term::Op(op) => match op {
            // Pipe chains are complex
            Op::Compose { operations, .. } if operations.len() > 1 => true,
            // Nested collections are complex
            Op::Sequence { operations, .. } if operations.len() > 2 => true,
            Op::Overlay { operations, .. } if operations.len() > 2 => true,
            Op::Choose { operations } if operations.len() > 2 => true,
            Op::Repeat { operations, .. } if operations.len() > 1 => true,
            Op::ModulateBy { .. } => true,
            Op::Lambda { .. } => true,
            _ => false,
        },
        _ => false,
    }
}

fn format_compose<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, operations: &[Term]) -> DocBuilder<'a, Arena<'a>> {
    if operations.is_empty() {
        return arena.nil();
    }

    // NOTE: Braced overtone syntax detection will be handled in Phase 4 using FormatTree

    if operations.len() == 1 {
        return format_term(arena, ctx, &operations[0]);
    }

    // Format with braces: { first | second | third }
    let inner = format_compose_inner(arena, ctx, operations);

    arena
        .text("{")
        .append(arena.line().append(inner).nest(2).group())
        .append(arena.line())
        .append(arena.text("}"))
        .group()
}

/// Format a compose chain WITHOUT braces - just the pipe-separated items.
/// Used when formatting inside collections where brackets provide grouping.
fn format_compose_inner<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, operations: &[Term]) -> DocBuilder<'a, Arena<'a>> {
    if operations.is_empty() {
        return arena.nil();
    }

    // NOTE: Braced overtone syntax detection will be handled in Phase 4 using FormatTree

    if operations.len() == 1 {
        return format_term(arena, ctx, &operations[0]);
    }

    // Format as: first | second | third (no braces)
    let formatted: Vec<_> = operations.iter().map(|t| format_term(arena, ctx, t)).collect();

    // Separator: pipe with spaces
    let sep = arena.text(" | ");

    arena.intersperse(formatted, sep)
}

/// Extract overtone values (fm, fa, g, p) from a Compose's operations
fn extract_overtone_values(operations: &[Term]) -> Option<(Rational64, Rational64, Rational64, Rational64)> {
    let mut f = None;
    let mut a = None;
    let mut g = None;
    let mut p = None;

    for op in operations {
        match op {
            Term::Op(Op::TransposeM { m, .. }) => f = Some(*m),
            Term::Op(Op::TransposeA { a: val, .. }) => a = Some(*val),
            Term::Op(Op::Gain { m, .. }) => g = Some(*m),
            Term::Op(Op::PanA { a: val, .. }) => p = Some(*val),
            _ => {}
        }
    }

    match (f, a, g, p) {
        (Some(f), Some(a), Some(g), Some(p)) => Some((f, a, g, p)),
        _ => None,
    }
}

fn format_listop<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, lop: &ListOp) -> DocBuilder<'a, Arena<'a>> {
    match lop {
        ListOp::Const { terms } => format_term_list(arena, ctx, terms),
        ListOp::Named { name } => arena.text(name.clone()),
        ListOp::ListOpIndexed { list_op, indices, direction } => {
            let base = format_listop(arena, ctx, list_op);
            let dir = match direction {
                weresocool_ast::Direction::Overlay => "O",
                weresocool_ast::Direction::Sequence => "S",
            };
            let idx_str = format!("{:?}", indices); // Simplified
            base.append(arena.text(format!("[{}]{}", idx_str, dir)))
        }
        ListOp::GenOp { generator } => format_genop(arena, ctx, generator),
        ListOp::Concat { listops } => {
            let parts: Vec<_> = listops.iter().map(|l| format_listop(arena, ctx, l)).collect();
            arena.intersperse(parts, arena.text(" ++ "))
        }
    }
}

fn format_genop<'a>(arena: &'a Arena<'a>, ctx: &FormatContext, gen: &GenOp) -> DocBuilder<'a, Arena<'a>> {
    match gen {
        GenOp::Named { name, .. } => arena.text(name.clone()),
        GenOp::Const { .. } => arena.text("<Generator>"),
        GenOp::Taken { generator, n, .. } => {
            let inner = format_genop(arena, ctx, generator);
            inner
                .append(arena.text(" |> Take "))
                .append(arena.text(n.to_string()))
        }
    }
}

use weresocool_ast::color::{ColorValue, CssOrHex};

/// Format a ColorValue back to source syntax
fn format_color_value<'a>(arena: &'a Arena<'a>, color: &ColorValue) -> DocBuilder<'a, Arena<'a>> {
    match color {
        ColorValue::Color(css_or_hex) => {
            let name = match css_or_hex {
                CssOrHex::Css(name) => name.clone(),
                CssOrHex::Hex(hex) => hex.clone(),
            };
            arena.text(format!("Color [{}]", name))
        }
        ColorValue::ColorSet { colors } => {
            if colors.len() == 1 {
                // Single color - format as hex
                let c = &colors[0];
                let hex = format!(
                    "#{:02x}{:02x}{:02x}",
                    (c.r * 255.0).round() as u8,
                    (c.g * 255.0).round() as u8,
                    (c.b * 255.0).round() as u8
                );
                arena.text(format!("Color [{}]", hex))
            } else {
                // Multiple colors - format as list
                let color_strs: Vec<String> = colors.iter().map(|c| {
                    format!(
                        "#{:02x}{:02x}{:02x}",
                        (c.r * 255.0).round() as u8,
                        (c.g * 255.0).round() as u8,
                        (c.b * 255.0).round() as u8
                    )
                }).collect();
                arena.text(format!("Color [{}]", color_strs.join(", ")))
            }
        }
    }
}


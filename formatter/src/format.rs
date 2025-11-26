//! Core formatting logic using pretty.rs

use crate::FormatConfig;
use num_rational::Rational64;
use pretty::{Arena, DocAllocator, DocBuilder};
use weresocool_ast::{
    FunDef, ListOp, GenOp, Op, Term,
    FmSyntax, FaSyntax, GainSyntax, LengthSyntax, PanMSyntax, PanASyntax,
    SeqSyntax, OverlaySyntax,
};
use weresocool_parser::{Init, ParsedComposition};

/// Format a complete parsed composition
pub fn format_composition(parsed: &ParsedComposition, config: &FormatConfig) -> String {
    let arena = Arena::new();
    let doc = format_composition_doc(&arena, parsed, config);
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
    _config: &FormatConfig,
) -> DocBuilder<'a, Arena<'a>> {
    let init_doc = format_init(arena, &parsed.init);

    // Get definitions from global scope
    let mut defs_docs: Vec<DocBuilder<'a, Arena<'a>>> = Vec::new();
    for (scope_name, scope) in parsed.defs.ops.iter() {
        // Only format global scope definitions
        if scope_name == "global" {
            for (name, term) in scope.iter() {
                let term_doc = format_term(arena, term);
                let def_doc = arena
                    .text(name.clone())
                    .append(arena.text(" = "))
                    .append(term_doc);
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
    if *r.denom() == 1 {
        // Integer or float-like
        let n = *r.numer();
        arena.text(n.to_string())
    } else {
        // Fraction
        arena.text(format!("{}/{}", r.numer(), r.denom()))
    }
}

fn format_term<'a>(arena: &'a Arena<'a>, term: &Term) -> DocBuilder<'a, Arena<'a>> {
    match term {
        Term::Op(op) => format_op(arena, op),
        Term::Nf(_) => arena.text("<NormalForm>"), // NormalForms shouldn't appear in source
        Term::FunDef(fun) => format_fundef(arena, fun),
        Term::Lop(lop) => format_listop(arena, lop),
        Term::Gen(gen) => format_genop(arena, gen),
    }
}

fn format_fundef<'a>(arena: &'a Arena<'a>, fun: &FunDef) -> DocBuilder<'a, Arena<'a>> {
    // name(arg1, arg2) = { ... }
    let args = arena.intersperse(
        fun.vars.iter().map(|v| arena.text(v.clone())),
        arena.text(", "),
    );
    let body = format_term(arena, &fun.term);

    arena
        .text(fun.name.clone())
        .append(arena.text("("))
        .append(args)
        .append(arena.text(") = "))
        .append(body)
}

fn format_op<'a>(arena: &'a Arena<'a>, op: &Op) -> DocBuilder<'a, Arena<'a>> {
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

        // Single rational parameter ops - preserve original syntax
        Op::TransposeM { m, syntax } => {
            let kw = match syntax {
                FmSyntax::Fm => "Fm",
                FmSyntax::Tm => "Tm",
            };
            format_single_rational_op(arena, kw, m)
        }
        Op::TransposeA { a, syntax } => {
            let kw = match syntax {
                FaSyntax::Fa => "Fa",
                FaSyntax::Ta => "Ta",
            };
            format_single_rational_op(arena, kw, a)
        }
        Op::PanM { m, syntax } => {
            let kw = match syntax {
                PanMSyntax::PanM => "PanM",
                PanMSyntax::Pm => "Pm",
            };
            format_single_rational_op(arena, kw, m)
        }
        Op::PanA { a, syntax } => {
            let kw = match syntax {
                PanASyntax::PanA => "PanA",
                PanASyntax::Pa => "Pa",
            };
            format_single_rational_op(arena, kw, a)
        }
        Op::Gain { m, syntax } => {
            let kw = match syntax {
                GainSyntax::Gain => "Gain",
                GainSyntax::Gm => "Gm",
            };
            format_single_rational_op(arena, kw, m)
        }
        Op::Length { m, syntax } => {
            let kw = match syntax {
                LengthSyntax::Length => "Length",
                LengthSyntax::Lm => "Lm",
            };
            format_single_rational_op(arena, kw, m)
        }
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

        // Collections - preserve original syntax
        Op::Sequence { operations, syntax } => {
            let kw = match syntax {
                SeqSyntax::Seq => "Seq",
                SeqSyntax::Sequence => "Sequence",
            };
            format_collection(arena, kw, operations)
        }
        Op::Overlay { operations, syntax } => {
            match syntax {
                OverlaySyntax::Overlay => format_collection(arena, "Overlay", operations),
                OverlaySyntax::O => format_overtone_overlay(arena, operations),
            }
        }
        Op::Choose { operations } => format_collection(arena, "Choose", operations),

        // Compose (pipe chains)
        Op::Compose { operations } => format_compose(arena, operations),

        // Repeat
        Op::Repeat { operations, count } => {
            if operations.len() == 1 {
                arena
                    .text("Repeat ")
                    .append(arena.text(count.to_string()))
            } else {
                let inner = format_term_list(arena, operations);
                arena
                    .text("Repeat ")
                    .append(arena.text(count.to_string()))
                    .append(arena.text(" "))
                    .append(inner)
            }
        }

        // ModulateBy
        Op::ModulateBy { operations, output } => {
            let inner = format_term_list(arena, operations);
            let base = arena.text("ModBy ").append(inner);
            if let Some(out) = output {
                let out_doc = format_term_list(arena, out);
                base.append(arena.text(" -> ")).append(out_doc)
            } else {
                base
            }
        }

        // Function call
        Op::FunctionCall { name, args } => {
            let args_doc = arena.intersperse(
                args.iter().map(|t| format_term(arena, t)),
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
            let body = format_term(arena, term);
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

        // WGSL - just output a placeholder since we don't have the original source
        Op::WGSL(id) => arena.text(format!("wgsl {{ /* block {} */ }}", id)),

        // Color operations
        Op::Color(id) => arena.text(format!("Color {}", id)),
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

        // Distortion effects
        Op::Wavefolder { threshold, stages, input_gain, output_gain } => {
            arena.text(format!(
                "Wavefolder {{ threshold: {}, stages: {}, input_gain: {}, output_gain: {} }}",
                threshold, stages, input_gain, output_gain
            ))
        }
        Op::SoftClip { threshold, input_gain, output_gain } => {
            arena.text(format!(
                "SoftClip {{ threshold: {}, input_gain: {}, output_gain: {} }}",
                threshold, input_gain, output_gain
            ))
        }
        Op::Overdrive { input_gain, output_gain } => {
            arena.text(format!(
                "Overdrive {{ input_gain: {}, output_gain: {} }}",
                input_gain, output_gain
            ))
        }
        Op::Bitcrusher { bits, input_gain, output_gain } => {
            arena.text(format!(
                "Bitcrusher {{ bits: {}, input_gain: {}, output_gain: {} }}",
                bits, input_gain, output_gain
            ))
        }
        Op::Tanh { input_gain, output_gain } => {
            arena.text(format!(
                "Tanh {{ input_gain: {}, output_gain: {} }}",
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
        Op::WithLengthRatioOf { main, with_length_of } => {
            let length_of_doc = format_term(arena, with_length_of);
            if let Some(m) = main {
                let main_doc = format_term(arena, m);
                main_doc
                    .append(arena.text(" | FitLength "))
                    .append(length_of_doc)
            } else {
                arena.text("FitLength ").append(length_of_doc)
            }
        }

        // Focus
        Op::Focus { name, main, op_to_apply } => {
            let main_doc = format_term(arena, main);
            let op_doc = format_term(arena, op_to_apply);
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
    name: &'static str,
    items: &[Term],
) -> DocBuilder<'a, Arena<'a>> {
    let inner = format_term_list(arena, items);

    // Smart grouping: try to fit on one line, otherwise break
    arena
        .text(name)
        .append(arena.text(" "))
        .append(inner)
}

/// Format overtone overlay using the O[...] notation with (f, a, g, p) tuples
fn format_overtone_overlay<'a>(
    arena: &'a Arena<'a>,
    items: &[Term],
) -> DocBuilder<'a, Arena<'a>> {
    if items.is_empty() {
        return arena.text("O []");
    }

    // Format each item - these should be Compose operations with 4 components
    let formatted_items: Vec<_> = items.iter().map(|t| format_overtone_tuple(arena, t)).collect();

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
fn format_overtone_tuple<'a>(arena: &'a Arena<'a>, term: &Term) -> DocBuilder<'a, Arena<'a>> {
    // An overtone tuple should be a Compose with 4 elements: TransposeM, TransposeA, Gain, PanA
    if let Term::Op(Op::Compose { operations }) = term {
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
    format_term(arena, term)
}

fn format_term_list<'a>(arena: &'a Arena<'a>, items: &[Term]) -> DocBuilder<'a, Arena<'a>> {
    if items.is_empty() {
        return arena.text("[]");
    }

    let formatted_items: Vec<_> = items.iter().map(|t| format_term(arena, t)).collect();

    // Create comma-separated list with smart line breaking
    let sep = arena.text(",").append(arena.line());
    let inner = arena.intersperse(formatted_items, sep);

    arena
        .text("[")
        .append(arena.line_().append(inner).nest(2).group())
        .append(arena.line_())
        .append(arena.text("]"))
        .group()
}

fn format_compose<'a>(arena: &'a Arena<'a>, operations: &[Term]) -> DocBuilder<'a, Arena<'a>> {
    if operations.is_empty() {
        return arena.nil();
    }

    if operations.len() == 1 {
        return format_term(arena, &operations[0]);
    }

    // Format as: { first | second | third }
    let formatted: Vec<_> = operations.iter().map(|t| format_term(arena, t)).collect();

    // Separator: line break then pipe
    let sep = arena.line().append(arena.text("| "));

    let inner = arena.intersperse(formatted, sep);

    arena
        .text("{")
        .append(arena.line().append(inner).nest(2).group())
        .append(arena.line())
        .append(arena.text("}"))
        .group()
}

fn format_listop<'a>(arena: &'a Arena<'a>, lop: &ListOp) -> DocBuilder<'a, Arena<'a>> {
    match lop {
        ListOp::Const { terms } => format_term_list(arena, terms),
        ListOp::Named { name } => arena.text(name.clone()),
        ListOp::ListOpIndexed { list_op, indices, direction } => {
            let base = format_listop(arena, list_op);
            let dir = match direction {
                weresocool_ast::Direction::Overlay => "O",
                weresocool_ast::Direction::Sequence => "S",
            };
            let idx_str = format!("{:?}", indices); // Simplified
            base.append(arena.text(format!("[{}]{}", idx_str, dir)))
        }
        ListOp::GenOp { generator } => format_genop(arena, generator),
        ListOp::Concat { listops } => {
            let parts: Vec<_> = listops.iter().map(|l| format_listop(arena, l)).collect();
            arena.intersperse(parts, arena.text(" ++ "))
        }
    }
}

fn format_genop<'a>(arena: &'a Arena<'a>, gen: &GenOp) -> DocBuilder<'a, Arena<'a>> {
    match gen {
        GenOp::Named { name, .. } => arena.text(name.clone()),
        GenOp::Const { .. } => arena.text("<Generator>"),
        GenOp::Taken { generator, n, .. } => {
            let inner = format_genop(arena, generator);
            inner
                .append(arena.text(" |> Take "))
                .append(arena.text(n.to_string()))
        }
    }
}

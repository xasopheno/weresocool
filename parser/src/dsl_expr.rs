//! The one expression sublanguage shared by the visual DSLs (see
//! `docs/DSL_STYLE.md`, Law 4).
//!
//! ONE AST ([`Expr`]), two backends: [`eval`] (CPU → `f32`, used by **draw**)
//! and [`to_wgsl`] (→ WGSL source, used by **warp**). Each DSL's grammar
//! produces a SUBSET of the variants — draw yields `Note`/`Stroke`, warp yields
//! `UserParam` (injected by its promote pass) — so the cross-domain variants are
//! unreachable in the other backend. The DSLs alias this type
//! (`pub use … Expr as WarpExpr` / `as DrawExpr`) so a shared AST can't drift:
//! adding a variant forces both backends to handle it.


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Per-note properties — draw only. Indices into the source `Op4D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteField {
    X,
    Y,
    Z,
    L,
    /// `note.t` — the note's onset (seconds). The per-note timestamp; was the
    /// capital `Time` atom before Law 4 folded it into the `note.*` family.
    T,
    Event,
    Voice,
}

/// Pure math functions (lowercase) — the Law-4 function set. Every name maps
/// 1:1 to a WGSL builtin, so `to_wgsl` is trivial. Distinct from the `Sin`
/// oscillator node (capital `Sin(freq)` = `sin(time*freq)`); `MathFn::Sin` is
/// the raw `sin(x)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MathFn {
    // unary
    Sin,
    Cos,
    Abs,
    Floor,
    Ceil,
    Sqrt,
    /// `exp(x)` — e^x. The falloff primitive: `exp(-Dist(bd) * 8.0)`.
    Exp,
    // binary
    Min,
    Max,
    Pow,
    /// `wrap(x, period)` — euclidean modulo (`x mod period`, always ≥0). The
    /// time-`mod` / loop primitive. No WGSL builtin, so `to_wgsl` special-cases it.
    Wrap,
    /// `step(edge, x)` — 0 below the edge, 1 at/above. The threshold primitive.
    Step,
    // ternary
    Clamp,
    Mix,
    /// `smoothstep(e0, e1, x)` — the smooth threshold; gates and masks.
    Smoothstep,
}

impl MathFn {
    /// The WGSL builtin name (also the source spelling).
    pub fn name(self) -> &'static str {
        match self {
            MathFn::Sin => "sin",
            MathFn::Cos => "cos",
            MathFn::Abs => "abs",
            MathFn::Floor => "floor",
            MathFn::Ceil => "ceil",
            MathFn::Sqrt => "sqrt",
            MathFn::Exp => "exp",
            MathFn::Min => "min",
            MathFn::Max => "max",
            MathFn::Pow => "pow",
            MathFn::Wrap => "wrap",
            MathFn::Clamp => "clamp",
            MathFn::Mix => "mix",
            MathFn::Step => "step",
            MathFn::Smoothstep => "smoothstep",
        }
    }
}

/// A scalar expression. The superset of what any DSL can parse; each grammar
/// produces its own subset (see the module docs).
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(f32),
    /// The live play clock — seconds of song time elapsed. The surface spelling
    /// is `clock` in every DSL (in warp it coincides with the frame's `time`,
    /// and lowers to `time`; in brush-wgsl the passthrough rewrites `clock` to
    /// the `song_time` uniform). Unlike a note's onset (`note.t`, static) this
    /// advances every frame, so an expression built on it moves over time.
    Clock,
    /// `note.<field>` — draw only (needs a concrete note; CPU-evaluated).
    Note(NoteField),
    /// `Stroke` — `0.0`→`1.0` along the current expansion — draw only.
    Stroke,
    /// `Time` — the ELEMENT'S OWN AGE. A stamp has one (seconds since it was
    /// laid down); a draw's marks do not age, and a warp frame's age is its
    /// clock. Host-specific like `Stroke` and `Hit`, and in the shared AST
    /// for the same reason: one grammar parses every DSL, and each host
    /// lowers the atoms it has.
    Age,
    /// `mod <name>` — loop the live clock over a named audio length. A parse-time
    /// placeholder: a later pass (with the length env) rewrites it to
    /// `wrap(Clock, Lit(seconds))`. The backends only see it if that pass was
    /// skipped, so they degrade gracefully (treat as the raw clock) rather than
    /// panic — this is a live instrument.
    LoopName(String),
    /// `cycle(<name>)` — your position through a named audio cycle as `0.0→1.0`
    /// (the time-sibling of `cycle(n)`). A parse-time placeholder; the length
    /// pass rewrites it to `wrap(clock, secs) / secs`. Degrades to `0.0` if the
    /// name is unknown. Drive cycle-locked color/shape with it.
    LoopPhase(String),
    /// `Sin(freq)` / `Sin(freq, amp)` — time-driven oscillation convenience
    /// (`sin(time*freq)*amp`). Distinct from `Call(MathFn::Sin, …)` (raw sine).
    Sin { freq: Box<Expr>, amp: Box<Expr> },
    /// A pure math function call — `sin(x)`, `clamp(x, lo, hi)`, etc.
    Call(MathFn, Vec<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// A literal promoted to a runtime-tweakable uniform slot — warp only,
    /// injected by `warp::promote`. Codegen emits `user_param(slot u)`.
    UserParam { slot: u32 },
    /// A FIELD read by name — warp only. The current value of a single
    /// medium channel at this pixel: a builtin ("r", "a", "sx"…) or a
    /// `state {}`-declared name ("water", "organism") that kintaro's
    /// resolve-fields pass canonicalises to a builtin before codegen.
    /// Lets gates, reaction terms and knobs read the medium itself:
    /// `Decay rgb (0.07 * (1.0 - smoothstep(0.03, 0.2, organism)))`.
    Field(String),
    /// `cycle([v0, v1, …])` — walk a list of values, one step per note,
    /// repeating: value = list[count % len]. Draw + brush-wgsl only (warp
    /// has no per-note count; its grammar never produces this).
    Cycle(Vec<Expr>),
    /// `rand(seed)` — deterministic per-note pseudo-random in `[0, 1)`, hashed
    /// from `count` and the seed. Same note → same value every frame (no
    /// flicker); different seeds are independent streams. Draw + brush-wgsl.
    Rand(Box<Expr>),
    /// `choose([v0, v1, …])` — random pick per note (the random sibling of
    /// `cycle([…])`): value = list[floor(rand * len)]. Draw + brush-wgsl.
    Choose(Vec<Expr>),
    /// A grammar-injected DEFAULT value for an omitted named arg — has NO
    /// counterpart in source text, so the promote pass must neither promote
    /// it nor advance its source-literal cursor over it. Everything else
    /// treats it exactly like `Lit`.
    DefaultLit(f32),
    /// warp-only — live per-brush hit envelope (attack on each new note
    /// event on that brush, exponential decay). Lowered to `hit(Nu)`,
    /// reading the `hit_data` uniform kintaro's envelope system feeds
    /// each frame. Composers write the PART'S NAME — `Hit(bd)` — and
    /// kintaro resolves it to a slot using the same table the runtime
    /// uses to record hits. Numeric form remains as an escape hatch.
    Hit(HitRef),
    /// The part's most recent onset x (warp uv space) — warp-only.
    HitX(HitRef),
    /// The part's most recent onset y (warp uv space) — warp-only.
    HitY(HitRef),
    /// Aspect-corrected distance from the current pixel to the part's most
    /// recent onset — warp-only. The field primitive under Ripple/Glow/
    /// Bulge: `exp(-Dist(bd) * 8.0)` is a falloff, `sin(Dist(bd) * 40.0)`
    /// is rings.
    Dist(HitRef),
    /// Seconds since the part's most recent onset — warp-only. Pairs with
    /// Dist for travelling waves: `sin(Dist(bd) * 40 - HitAge(bd) * 6)`.
    HitAge(HitRef),
}

/// How a hit channel is referenced in source: by the part's NAME (the
/// normal way — `Hit(bd)`) or by a raw slot index (escape hatch).
/// Kintaro's resolve pass rewrites every `Name` to an `Idx` before
/// codegen, using the exact name→slot table the runtime bump site uses.
#[derive(Debug, Clone, PartialEq)]
pub enum HitRef {
    Idx(u32),
    Name(String),
}

impl HitRef {
    /// The resolved slot; unresolved names fall to the dead top slot
    /// (nothing ever bumps it) so a typo'd name is inert, not chaotic.
    pub fn idx(&self) -> u32 {
        match self {
            HitRef::Idx(i) => *i,
            HitRef::Name(_) => 127,
        }
    }
}

// The draw-backend evaluator (`eval` against a concrete `Op4D` note) lives
// in kintaro (`dsl_expr_eval`) — `Op4D` is a render-side type from
// weresocool_core, which this crate must not depend on (cycle).

/// **warp backend** — lower to a WGSL expression string.
pub fn to_wgsl(e: &Expr) -> String {
    to_wgsl_with(e, &|_| None)
}

/// The same lowering, with a HOOK the caller gets to answer with first.
///
/// A substrate knows things this crate cannot. `slope(c)` needs to know which
/// TEXTURE a channel lives in; `Cycle`/`Choose`/`Rand` need a step index. Those
/// live in kintaro, so kintaro used to special-case them before delegating here
/// for everything else — which worked only when the special node was the WHOLE
/// expression. The moment it appeared inside arithmetic, as in
///
///     Decay RGB (0.94 - slope(h) * 30.0)
///
/// the top node was a subtraction, the whole tree came here, and this walker
/// recursed with ITSELF — so the nested `slope` fell through to the unresolved
/// -field arm and silently became 0.0. Two walkers, one of them blind.
///
/// The hook is consulted at EVERY node on the way down, so a substrate's nodes
/// survive at any depth. Returning `None` means "you handle it".
pub fn to_wgsl_with(e: &Expr, hook: &dyn Fn(&Expr) -> Option<String>) -> String {
    if let Some(s) = hook(e) {
        return s;
    }
    let to_wgsl = |e: &Expr| to_wgsl_with(e, hook);
    match e {
        Expr::Lit(n) => {
            // inf/NaN would print as `inf`/`NaN` — invalid WGSL that only fails
            // at the runtime shader swap. Emit a benign zero instead.
            if !n.is_finite() {
                return "0.0".into();
            }
            // WGSL requires a decimal for f32 literals when there's ambiguity.
            if n.fract() == 0.0 {
                format!("{:.1}", n)
            } else {
                format!("{}", n)
            }
        }
        // warp is a fullscreen compositor with no per-element birth, so its
        // `params.time` (emitted as `time`) already IS the live play clock —
        // `clock` lowers to `time` here. (The brush-wgsl live clock is a
        // separate identifier, `song_time`, surfaced by weresocool's own
        // passthrough, not this backend.)
        Expr::Clock => "time".into(),
        // a compositor has no per-element birth: its age IS its clock
        Expr::Age => "time".into(),
        Expr::Sin { freq, amp } => {
            format!("(sin(time * ({})) * ({}))", to_wgsl(freq), to_wgsl(amp))
        }
        Expr::Call(MathFn::Wrap, args) => {
            // WGSL `%` is C-style remainder (sign follows dividend), not euclid.
            // Emit the standard positive-modulo idiom so the loop never goes < 0.
            let x = args.first().map(to_wgsl).unwrap_or_else(|| "0.0".into());
            let p = args.get(1).map(to_wgsl).unwrap_or_else(|| "1.0".into());
            format!("((({x}) % ({p}) + ({p})) % ({p}))")
        }
        Expr::Call(f, args) => {
            let a: Vec<String> = args.iter().map(to_wgsl).collect();
            format!("{}({})", f.name(), a.join(", "))
        }
        Expr::Bin(op, l, r) => {
            let sym = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
            };
            format!("(({}) {} ({}))", to_wgsl(l), sym, to_wgsl(r))
        }
        Expr::UserParam { slot } => format!("user_param({}u)", slot),
        // Field reads — canonical channel names (kintaro's resolve-fields
        // pass rewrote `state {}` names to these before codegen). Warp-only:
        // `color`/`state` are the warp step's working variables.
        Expr::Field(name) => match name.as_str() {
            "R" => "color.r".into(),
            "G" => "color.g".into(),
            "B" => "color.b".into(),
            "A" => "color.a".into(),
            "Sx" => "state.x".into(),
            "Sy" => "state.y".into(),
            "Sz" => "state.z".into(),
            "Sw" => "state.w".into(),
            // The l-basis (seconds per base length unit) — codegen emits a
            // `let LBase = <value>;` prologue into every warp fn.
            "LBase" => "LBase".into(),
            other => {
                eprintln!("[warp] unresolved field `{}` in expression — emitting 0.0", other);
                "0.0".into()
            }
        },
        Expr::DefaultLit(n) => {
            if n.fract() == 0.0 && n.abs() < 1e6 {
                format!("{:.1}", n)
            } else {
                format!("{}", n)
            }
        }
        // Draw-only (evaluated on the CPU); the warp grammar never
        // produces these, so the arms are safe inert fallbacks.
        Expr::Cycle(_) | Expr::Rand(_) | Expr::Choose(_) | Expr::Age => "0.0".to_string(),
        Expr::Hit(ch) => format!("hit({}u)", ch.idx()),
        Expr::HitX(ch) => format!("hit_x({}u)", ch.idx()),
        Expr::HitY(ch) => format!("hit_y({}u)", ch.idx()),
        Expr::Dist(ch) => format!("k_hit_dist({}u, uv)", ch.idx()),
        Expr::HitAge(ch) => format!("hit_age({}u)", ch.idx()),
        // Unresolved loop placeholders (should be rewritten pre-codegen); the
        // warp live clock is `time`, so degrade to that.
        Expr::LoopName(_) => "time".into(),
        Expr::LoopPhase(_) => "0.0".into(),
        // draw-only; the warp grammar never produces these.
        Expr::Note(_) | Expr::Stroke => {
            unreachable!("draw-only expr (note.*/Stroke) in a warp expression")
        }
    }
}

/// Best-effort **constant fold** — `Some(v)` if the expression has no runtime
/// references. `Time`/`Note`/`Stroke`/`Sin`/`UserParam` make it `None`; `Bin`
/// and pure `Call`s fold when all their operands do. Used by the FitLength
/// rescale and the stability lint (both need static values).
/// Does this expression read `Stroke`?
///
/// `Stroke` is the only atom whose value differs BETWEEN EMITS of one note —
/// everything else (note fields, the clock, rand) is constant across the set.
/// So it is also the only reason an op has to evaluate its argument per emit
/// instead of once, and the difference is worth knowing: a 2000-mark set would
/// otherwise pay for 2000 evaluations of an expression that cannot change.
pub fn uses_stroke(e: &Expr) -> bool {
    match e {
        Expr::Stroke => true,
        Expr::Sin { freq, amp } => uses_stroke(freq) || uses_stroke(amp),
        Expr::Call(_, args) | Expr::Cycle(args) | Expr::Choose(args) => {
            args.iter().any(uses_stroke)
        }
        Expr::Bin(_, a, b) => uses_stroke(a) || uses_stroke(b),
        Expr::Rand(a) => uses_stroke(a),
        _ => false,
    }
}

pub fn as_const(e: &Expr) -> Option<f64> {
    match e {
        Expr::Lit(v) => Some(*v as f64),
        Expr::DefaultLit(v) => Some(*v as f64),
        Expr::Cycle(_) | Expr::Rand(_) | Expr::Choose(_) | Expr::Age => None,
        Expr::Bin(op, l, r) => {
            let (l, r) = (as_const(l)?, as_const(r)?);
            Some(match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => {
                    if r == 0.0 {
                        return None;
                    }
                    l / r
                }
            })
        }
        Expr::Call(f, args) => {
            let a = args.iter().map(as_const).collect::<Option<Vec<f64>>>()?;
            let g = |i: usize| a.get(i).copied().unwrap_or(0.0);
            Some(match f {
                MathFn::Sin => g(0).sin(),
                MathFn::Cos => g(0).cos(),
                MathFn::Abs => g(0).abs(),
                MathFn::Floor => g(0).floor(),
                MathFn::Ceil => g(0).ceil(),
                MathFn::Sqrt => g(0).max(0.0).sqrt(),
                MathFn::Exp => g(0).exp(),
                MathFn::Min => g(0).min(g(1)),
                MathFn::Max => g(0).max(g(1)),
                MathFn::Pow => g(0).powf(g(1)),
                MathFn::Wrap => {
                    let p = g(1);
                    if p > 0.0 { g(0).rem_euclid(p) } else { g(0) }
                }
                MathFn::Clamp => g(0).clamp(g(1).min(g(2)), g(1).max(g(2))),
                MathFn::Mix => g(0) + (g(1) - g(0)) * g(2),
                MathFn::Step => if g(1) >= g(0) { 1.0 } else { 0.0 },
                MathFn::Smoothstep => {
                    let t = ((g(2) - g(0)) / (g(1) - g(0)).max(1e-9)).clamp(0.0, 1.0);
                    t * t * (3.0 - 2.0 * t)
                }
            })
        }
        // Runtime references — not statically knowable.
        Expr::Clock
        | Expr::LoopName(_)
        | Expr::LoopPhase(_)
        | Expr::Note(_)
        | Expr::Stroke
        | Expr::Sin { .. }
        | Expr::UserParam { .. }
        | Expr::Hit(_)
        | Expr::HitX(_)
        | Expr::HitY(_)
        | Expr::Dist(_)
        | Expr::HitAge(_)
        | Expr::Field(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_folds_bin_and_call() {
        use Expr::*;
        assert_eq!(as_const(&Bin(BinOp::Mul, Box::new(Lit(3.0)), Box::new(Lit(4.0)))), Some(12.0));
        assert_eq!(as_const(&Call(MathFn::Sqrt, vec![Lit(9.0)])), Some(3.0));
        assert_eq!(
            as_const(&Call(MathFn::Clamp, vec![Lit(2.0), Lit(0.0), Lit(1.0)])),
            Some(1.0)
        );
        // runtime refs poison the fold
        assert_eq!(as_const(&Call(MathFn::Abs, vec![Clock])), None);
    }

    #[test]
    fn to_wgsl_function_set() {
        use Expr::*;
        assert_eq!(to_wgsl(&Call(MathFn::Sin, vec![Clock])), "sin(time)");
        assert_eq!(
            to_wgsl(&Call(MathFn::Clamp, vec![Clock, Lit(0.0), Lit(1.0)])),
            "clamp(time, 0.0, 1.0)"
        );
        // variadic max is stored (and emitted) as nested binary calls.
        let m = Call(
            MathFn::Max,
            vec![Call(MathFn::Max, vec![Lit(1.0), Lit(2.0)]), Lit(3.0)],
        );
        assert_eq!(to_wgsl(&m), "max(max(1.0, 2.0), 3.0)");
    }

    #[test]
    fn clock_and_wrap() {
        use Expr::*;
        // Clock is runtime, so no const fold; warp's params.time is the live
        // clock, so Clock lowers to `time`. (Runtime eval assertions live in
        // kintaro's dsl_expr_eval tests, next to the evaluator.)
        assert_eq!(as_const(&Clock), None);
        assert_eq!(to_wgsl(&Clock), "time");

        // wrap(7, 3) == 1 (euclid), folds statically, and lowers to positive-%.
        let w = Call(MathFn::Wrap, vec![Lit(7.0), Lit(3.0)]);
        assert_eq!(as_const(&w), Some(1.0));
        assert_eq!(to_wgsl(&w), "(((7.0) % (3.0) + (3.0)) % (3.0))");
    }
}

//! Overlay WIP layers into the composition at process time — WITHOUT touching
//! the user's `.socool`.
//!
//! For each non-muted layer we (a) seed its frozen events into the `Perform`
//! recordings registry, and (b) transform the source in-memory: append a def
//! `__layer<id> = { <palette_sound> | Perform("__layer<id>_ev") }` and overlay
//! the layers into `main`. The palette sound decorates each recorded note (sing
//! a melody through `fifths` → parallel fifths in that color); the layers flow
//! through the full brush/warp/draw pipeline like any voice.

use num_rational::Rational64;
use weresocool_ast::{NormalForm, OscType, PointOp, RecordingRegistry};

use super::store::{DawStore, Event};

/// A def to splice into the source and overlay into `main`.
#[derive(Clone)]
pub struct InjectedDef {
    pub name: String,
    /// Full `{ … }` body.
    pub body: String,
}

/// What to inject: registry entries for `Perform` resolution + defs to splice
/// into the source and overlay into `main`.
#[derive(Default, Clone)]
pub struct LayerInjection {
    pub registry: RecordingRegistry,
    pub defs: Vec<InjectedDef>,
    /// The palette slot name (e.g. `perf`, or `daw` when the palette is
    /// unnamed) — the token the user drops into `main` where layers sit.
    pub slot: String,
    /// True when there's anything to hear (layers and/or an armed monitor).
    /// False → the injection is only the always-defined (silent) slot def,
    /// needed so a placed slot token still parses.
    pub has_content: bool,
}

/// A loop-length anchor def (= `main` with the slot neutralized) so the monitor
/// can `FitLength` to the whole loop without depending on itself (no circular
/// FitLength).
const FIT_ANCHOR: &str = "__daw_len";

/// Build the injection from the store: a `Perform` voice per non-muted layer,
/// plus a live `Follow` monitor for the armed palette sound (if any). The layers
/// keep their NATURAL timing (loop-aligned by leading silence); the monitor is
/// one long note spanning the loop via `FitLength __daw_len`. All are collected
/// into the `<slot>` overlay def that the user places in `main`.
pub fn build_layer_injection(store: &DawStore, slot: &str) -> LayerInjection {
    let mut inj = LayerInjection { slot: slot.to_string(), ..Default::default() };
    let solo = store.any_soloed();
    let mut voices: Vec<String> = Vec::new();

    for layer in store.layers() {
        // Solo wins: if anything is soloed, only soloed layers sound.
        let active = if solo { layer.soloed } else { !layer.muted };
        if !active {
            continue;
        }
        let def_name = format!("__layer{}", layer.id);
        let ev_name = format!("{}_ev", def_name);
        inj.registry
            .insert(ev_name.clone(), events_to_normalform(&layer.events));
        inj.defs.push(InjectedDef {
            name: def_name.clone(),
            // The frozen contour through the sound at its NATURAL timing (what/
            // when you sang), loop-aligned by leading silence in the events. NO
            // FitLength — wgsl is per-brush-age, so it can't help; and stretching
            // fights the recorded tempo. Placement in `main` (at the slot) is what
            // makes the layer inherit the piece's look/level.
            // Rendered at UNITY and tagged `#layer<id>`. The layer's fader
            // volume is NOT baked in here — it's applied at runtime as a
            // per-voice `gain_mul` (see `apply_live_layer_gains_system`), so
            // dragging the volume slider costs a gain multiply, not a re-parse
            // + re-render. The `#layer<id>` tag is the provenance the runtime
            // fader matches voices on.
            body: format!(
                "{{ {} | Perform(\"{}\") | #layer{} }}",
                layer.palette_sound, ev_name, layer.id
            ),
        });
        voices.push(def_name);
    }

    // Live monitor: the armed sound following the mic, one note spanning the
    // whole loop. (The engine slices a long note per audio buffer, so it both
    // follows continuously and emits a stream of brushes — no Repeat needed.)
    //
    // NOT part of the slot: the monitor renders on its own DIRECT render
    // manager (rm_mon, low-latency mic path), while the slot's frozen layers
    // ride the pre-rendered main comp (rm_main). This def is unreferenced by
    // `main` — the main comp never renders it; `monitor_source` builds rm_mon's
    // comp from it. Style the monitor via the palette sound itself (its Color /
    // wgsl); it doesn't inherit main's pipe.
    let armed = store.armed_sound().is_some();
    if let Some(sound) = store.armed_sound() {
        inj.defs.push(InjectedDef {
            name: MONITOR_DEF.to_string(),
            // `#tag` marks every op flowing through, so the monitor's voices can
            // be picked back out of `__monitor_main` (main evaluated with the
            // slot bound to the monitor) — provenance for the two-path split.
            body: format!(
                "{{ {sound} | Follow {{ _ -> {{F, G}} }} | FitLength {FIT_ANCHOR} | #{MONITOR_TAG} }}"
            ),
        });
    }

    // The `<slot>` def — the single handle the user drops into `main`. Always
    // defined (silent when no layers) so a placed slot token parses even before
    // the first recording.
    inj.has_content = !voices.is_empty() || armed;
    inj.defs.push(InjectedDef {
        name: slot.to_string(),
        body: if voices.is_empty() {
            "{ Fm 0 }".to_string()
        } else {
            format!("{{ Overlay [ {} ] }}", voices.join(", "))
        },
    });

    inj
}

/// The live-monitor def name (rendered by rm_mon, not the main comp).
pub const MONITOR_DEF: &str = "__monitor";

/// `main` evaluated with the slot bound to the monitor — so the monitor gets
/// EXACTLY the treatment a frozen layer will get at that position (downstream
/// wgsl / gain / pitch ops). rm_mon renders only the `#daw_monitor`-tagged
/// voices filtered out of this def's NormalForm.
pub const MONITOR_MAIN: &str = "__monitor_main";

/// The `#tag` marking the monitor's ops inside `__monitor_main` (provenance —
/// how build_monitor_voices separates the monitor from the rest of main).
pub const MONITOR_TAG: &str = "daw_monitor";

/// Pitches within this many cents merge into one held note (vibrato tolerance).
const MERGE_CENTS: f64 = 70.0;

/// Gains within this relative band merge; a bigger jump starts a new event so
/// the performed dynamics (swells, accents) survive into playback instead of
/// being averaged flat — playback should look/sound like what you performed.
const MERGE_GAIN_REL: f64 = 0.25;

/// Run-length-merge a per-block event stream into held notes: consecutive
/// events with the same voiced/silent state, pitch within `MERGE_CENTS`, and
/// gain within `MERGE_GAIN_REL` collapse into one (lengths summed, gain
/// meaned). Keeps a captured loop from being thousands of ~1.5ms notes while
/// preserving the pitch AND dynamics contour.
pub fn merge_events(events: Vec<Event>) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut counts: Vec<usize> = Vec::new();
    for e in events {
        match out.last_mut() {
            Some(prev) if same_note(prev, &e, *counts.last().unwrap()) => {
                prev.secs += e.secs;
                prev.gain += e.gain;
                *counts.last_mut().unwrap() += 1;
            }
            _ => {
                out.push(e);
                counts.push(1);
            }
        }
    }
    for (e, c) in out.iter_mut().zip(counts) {
        if c > 1 {
            e.gain /= c as f64;
        }
    }
    out
}

/// `a.gain` is a RUNNING SUM over `a_count` merged blocks (meaned at the end).
fn same_note(a: &Event, b: &Event, a_count: usize) -> bool {
    let a_mean = a.gain / a_count.max(1) as f64;
    let a_sil = a_mean <= 0.0;
    let b_sil = b.gain <= 0.0;
    if a_sil || b_sil {
        return a_sil && b_sil;
    }
    if a.fm <= 0.0 || b.fm <= 0.0 {
        return false;
    }
    if 1200.0 * (a.fm / b.fm).log2().abs() > MERGE_CENTS {
        return false;
    }
    // Same pitch but a real dynamics change → new event (keep the contour).
    let hi = a_mean.max(b.gain);
    let lo = a_mean.min(b.gain);
    (hi - lo) / hi <= MERGE_GAIN_REL
}

/// Round a float to a rational with a bounded denominator. `approximate_float`
/// returns the EXACT f64 rational (denominators up to 2^52), which overflows
/// `i64` the moment lengths/freqs are multiplied during normalization.
fn rat(x: f64, denom: i64) -> Rational64 {
    Rational64::new((x * denom as f64).round() as i64, denom)
}

/// Rebuild a layer's `NormalForm` from its stored events.
pub fn events_to_normalform(events: &[Event]) -> NormalForm {
    let mut ops: Vec<PointOp> = Vec::with_capacity(events.len());
    for e in events {
        let voiced = e.gain > 0.0;
        ops.push(PointOp {
            fm: if voiced { rat(e.fm, 1000) } else { Rational64::from_integer(1) },
            g: rat(e.gain, 1000),
            l: rat(e.secs.max(0.0), 1000),
            osc_type: OscType::Sine { pow: None },
            ..Default::default()
        });
    }
    let total: Rational64 = ops.iter().map(|o| o.l).sum();
    NormalForm {
        operations: vec![ops],
        length_ratio: total,
        start_at: None,
    }
}

/// Splice the layer defs into `source` and connect them at the `<slot>` the user
/// placed in `main` (so they inherit whatever wraps that position — level, warp,
/// wgsl). If the slot isn't in `main`, fall back to appending it as a sibling.
/// Returns the source unchanged if there are no layers (or `main` can't be found).
pub fn apply_injection(source: &str, inj: &LayerInjection) -> String {
    let Some((open, close)) = super::push::find_def_braces(source, "main") else {
        return source.to_string();
    };
    let slot = &inj.slot;
    let body = &source[open + 1..close];

    // Nothing to hear AND no slot placed → leave the source untouched. (With a
    // placed slot we still inject the silent slot def so `main` parses.)
    if inj.defs.is_empty() || (!inj.has_content && !contains_token(body, slot)) {
        return source.to_string();
    }

    // Loop-length anchor for the monitor — `main` with the slot neutralized to
    // `None`, so the monitor's FitLength doesn't include itself (no circular).
    let fit_ref = replace_token(body, slot, "None");

    let mut out = String::with_capacity(source.len() + 256);
    out.push_str(&source[..open]);
    if contains_token(body, slot) {
        // The user placed the slot — fill it, leave `main` exactly as written so
        // the layers inherit exactly the ops that wrap that spot.
        out.push_str(&source[open..=close]);
    } else {
        // Fallback: append as a sibling. Works, but the layers won't inherit any
        // mid-pipe wgsl/warp — placing the slot explicitly is the correct path.
        eprintln!(
            "[daw] '{slot}' not found in main — appending layers as a sibling. \
             Put `{slot}` in main where you want them (e.g. inside the wgsl pipe) \
             for matching visuals."
        );
        out.push_str(&format!("{{ Overlay [ ({}), {slot} ] }}", body.trim()));
    }
    out.push_str(&source[close + 1..]);

    // Append the loop-length anchor + the injected defs (`<slot>`, layers,
    // monitor). Order vs main doesn't matter — defs resolve at normalize time.
    out.push('\n');
    out.push_str(&format!("{FIT_ANCHOR} = {{ {} }}\n", fit_ref.trim()));
    for d in &inj.defs {
        out.push_str(&format!("{} = {}\n", d.name, d.body));
    }

    // When armed, also emit `__monitor_main` = main with the slot bound to the
    // monitor (layers swapped out). rm_mon renders the tagged voices from this,
    // so what you hear/see while monitoring IS what the frozen layer will
    // get at that position.
    if inj.defs.iter().any(|d| d.name == MONITOR_DEF) {
        let mon_body = if contains_token(body, slot) {
            replace_token(body, slot, MONITOR_DEF)
        } else {
            format!("Overlay [ ({}), {MONITOR_DEF} ]", body.trim())
        };
        out.push_str(&format!("{MONITOR_MAIN} = {{ {} }}\n", mon_body.trim()));
    }
    out
}

/// Whether `name` appears as a standalone identifier in `text`.
pub fn contains_token(text: &str, name: &str) -> bool {
    let b = text.as_bytes();
    let mut from = 0;
    while let Some(rel) = text[from..].find(name) {
        let at = from + rel;
        let end = at + name.len();
        let prev_ok = at == 0 || !is_ident(b[at - 1]);
        let next_ok = b.get(end).map_or(true, |&c| !is_ident(c));
        if prev_ok && next_ok {
            return true;
        }
        from = end;
    }
    false
}

/// Replace standalone identifier `name` with `with` (word-boundary, so it won't
/// touch `__daw_main` when replacing `daw`).
fn replace_token(text: &str, name: &str, with: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text[i..].starts_with(name) {
            let prev_ok = i == 0 || !is_ident(b[i - 1]);
            let next = b.get(i + name.len()).copied();
            let next_ok = next.map_or(true, |c| !is_ident(c));
            if prev_ok && next_ok {
                out.push_str(with);
                i += name.len();
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_ident(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric() || c == b'.'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inj_with(slot: &str, defs: Vec<InjectedDef>) -> LayerInjection {
        LayerInjection {
            registry: RecordingRegistry::new(),
            defs,
            slot: slot.to_string(),
            has_content: true,
        }
    }

    #[test]
    fn slot_absent_falls_back_to_sibling() {
        let inj = inj_with(
            "daw",
            vec![InjectedDef {
                name: "__layer1".to_string(),
                body: "{ fifths | Perform(\"__layer1_ev\") }".to_string(),
            }],
        );
        let src = "fifths = { Fm 1 }\nmain = { thing1 }\n";
        let out = apply_injection(src, &inj);
        // No slot in main → sibling fallback.
        assert!(out.contains("Overlay [ (thing1), daw ]"));
        assert!(out.contains("__daw_len = {"));
        assert!(out.contains("__layer1 = { fifths | Perform(\"__layer1_ev\") }"));
    }

    #[test]
    fn empty_injection_is_passthrough() {
        let src = "main = { x }\n";
        assert_eq!(apply_injection(src, &LayerInjection::default()), src);
    }

    #[test]
    fn user_placed_slot_left_in_main() {
        // Named palette slot `perf`, placed inside the pipe — main kept verbatim
        // so the layers inherit exactly the ops after `perf`.
        let inj = inj_with(
            "perf",
            vec![InjectedDef {
                name: "perf".to_string(),
                body: "{ Overlay [ __layer1 ] }".to_string(),
            }],
        );
        let src = "main = { Overlay [ thing1, perf ] | impasto }\n";
        let out = apply_injection(src, &inj);
        assert!(out.contains("main = { Overlay [ thing1, perf ] | impasto }"));
        // Loop-length anchor excludes the slot (→ None).
        assert!(out.contains("__daw_len = { Overlay [ thing1, None ] | impasto }"));
    }

    #[test]
    fn messy_event_floats_stay_bounded() {
        // Realistic mic values (not clean fractions) must not produce huge
        // rationals (which overflowed i64 with approximate_float).
        let events: Vec<Event> = (0..40)
            .map(|i| Event {
                fm: 1.0 + (i as f64) * 0.0173,
                gain: 0.4567 + (i as f64) * 0.001,
                secs: 0.0029 * (i as f64 + 1.0),
            })
            .collect();
        let nf = events_to_normalform(&events);
        for op in &nf.operations[0] {
            assert!(*op.fm.denom() <= 1000 && op.fm.numer().abs() < 100_000);
            assert!(*op.l.denom() <= 1000);
        }
    }
}

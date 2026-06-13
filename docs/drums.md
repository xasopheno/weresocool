# Drums

WereSoCool has five synthesized drums — `Kick`, `Snare`, `HiHat` (plus
`OpenHat`, the open variant), `Clap`, and `Rimshot` — built on physical
modeling: Karplus-Strong
waveguides, 22-mode Bessel cymbal physics, stateful resonant filters,
velocity-dependent timbre, and analog-style saturation and compression.

The design goal is **zero or one decision gets a great sound**:

```
Kick                      -- the signature voicing, ready to use
Kick 808                  -- a named preset
Kick 808 { tune: 1/2 }    -- preset + targeted overrides
```

Drums are ordinary oscillators: they respond to all language ops.
`bd | Fm 1/2` drops a kick an octave. `Gm` is **velocity** — it changes
timbre, not just level (soft hits round off, hard hits crack and saturate).

## Presets

Every preset is loudness-calibrated to within ~0.1 dB LUFS of its family,
so switching presets never blows up a mix. Unknown preset names are a parse
error that lists the available names.

### Kick

| Preset | Character |
|---|---|
| `wsc` (default) | The signature voicing: waveguide body, two-stage pitch knee, warm asymmetric saturation, punchy hump |
| `808` | Long sub boom — sine-dominant body, slow settle, almost no click |
| `909` | Dance-floor punch — single-exponential pitch drop, hard beater click, tight decay |
| `knock` | Trap kick — 808 bones, shorter, driven hard into the tape stage; knocks on small speakers |
| `acoustic` | Waveguide-dominant with wooden shell ring; a played drum |
| `dust` | Lofi — dark, soft, short; the kick on a worn record |

### Snare

| Preset | Character |
|---|---|
| `wsc` (default) | Crisp crack, balanced 4-band wires, lively velocity response |
| `808` | Tight dark snap — shaped noise burst more than drum |
| `909` | Tonal body + bright wide burst, punchy compression |
| `trap` | Bright, short, aggressive mid-band crack — the modern produced snap |
| `brush` | Head-dominant, long shell, soft beater — acoustic-adjacent |
| `dust` | Lofi — dark wires, low crack; sampled-off-vinyl |

### HiHat / OpenHat

| Preset | Character |
|---|---|
| `wsc` (default) | Balanced 22-mode shimmer and air |
| `808` | Dark metallic square-bank character — low modes favored, less air |
| `909` | Bright noisy sizzle — air-forward, crisp attack |
| `trap` | Tight tick built for rolls (`hh \| Lm 1/4 \| Repeat 4`) |
| `acoustic` | Looser live pair — longer ring, more modal beating |
| `dust` | Lofi — airless, dark, papery |

### Clap

| Preset | Character |
|---|---|
| `wsc` (default) | Balanced burst train + room tail |
| `808` | The iconic spread clap — slower bursts, resonant ~1 kHz, generous tail |
| `909` | Tighter and noisier — three fast bursts, brighter, shorter |
| `trap` | Bright layered snap — wide top band, sharp bursts |
| `dust` | Lofi — dark, papery, short |

Every clap's burst rhythm is randomized per hit — no two claps are identical.

### Rimshot

| Preset | Character |
|---|---|
| `wsc` (default) | Balanced woody-metallic "tock" |
| `808` | The tonal ping — higher, purer, slightly longer |
| `909` | Brighter click-forward rim — more stick, less ring |
| `acoustic` | The woody side-stick — low, fast, soft click |

## The parameter model

Three layers, each overriding the one below:

1. **Explicit params** — `Kick { click_freq: 2000 }` pins a value exactly.
2. **Macro knobs** (0–1) — broad-stroke character controls. They *bend the
   preset*: each preset defines what its macros mean, so `attack: 0.8` on
   `Kick 808` gives you the hardest 808 attack, not a 909.
3. **The preset** — supplies everything you didn't specify.

Macros per drum:

| Drum | Macros |
|---|---|
| Kick | `attack` (soft→clicky), `body` (thin→fat), `tone` (dark→bright), `length` (tight→boomy) |
| Snare | `attack` (soft→cracking), `wires` (dry→sizzly), `tone` (dark→bright), `length` (tight→ringy) |
| HiHat | `attack` (soft→clicky), `metal` (dull→shimmery), `length` (choked→open) |
| Clap | `attack` (soft→spitty), `spread` (tight→wide), `tone` (dark→bright), `length` (dry→roomy) |
| Rimshot | `tone` (woody→metallic), `length` (tick→ring) |

Presets also control *internal* engine behavior that has no parameter at
all — the kick's pitch-envelope shape (two-stage 808 knee vs. straight 909
drop), the cymbal's size and spectral tilt, the snare-wire resonance
frequencies, compressor settings. This is why `Kick 909` is a genuinely
different instrument from `Kick 808`, not the same machine with different
knob positions.

## Specific overrides (full reference)

**Kick**: `tune`, `pitch_decay` (s), `pitch_range` (start multiple),
`amp_decay` (s), `saturation`, `hump` (punch pump depth), `shell` (wood
ring), `ks_mix` (waveguide blend), `click_amount`, `click_freq` (Hz),
`velocity_tilt` (how much Gm changes timbre).

**Snare**: `tune`, `shell_decay` (s), `wire_decay` (s), `wire_mix`,
`shell_tune` (bottom-head ratio), `attack_amount` (beater),
`shell_pitch_decay`, `shell_pitch_range`, `head_damping_ratio`,
`saturation`, `crack`, `crack_freq` (Hz), `ks_mix`, `velocity_tilt`.

**HiHat/OpenHat**: `tune`, `decay_rate` (per second), `shimmer`,
`brightness`, `attack_amount`, `ping_amount` (stick-on-bell tone),
`pitch_drop`, `velocity_tilt`.

**Clap**: `tune`, `saturation`, `velocity_tilt`.

**Rimshot**: `tune`, `attack_amount` (click), `velocity_tilt`.

## Kits

`kits/` contains ready-made kit files (`kit_808`, `kit_909`, `kit_trap`,
`kit_acoustic`, `kit_industrial`, `kit_minimal`) — each defines `bd`, `sn`,
`hh`, `oh`, `cp`, `rs` with mix staging, ready to copy or adapt. `kits/showcase.socool`
plays every preset in sequence.

A kit is just named definitions, so making your own is one line per drum:

```
bd = { Kick knock | Gm 0.65 }
sn = { Snare trap { crack_freq: 4500 } | Fm 4 | Gm 0.5 }
hh = { HiHat trap | Fm 55 | Gm 0.2 }

beat = {
    Overlay [
        Seq [bd, x, x, x, bd, x, x, x],
        Seq [x, x, sn, x, x, x, sn, x],
        Seq [hh, hh, hh, hh | Gm 0.3, hh, hh, hh, hh | Gm 0.6]
    ]
}
```

## Engine notes (for the curious)

- Drums are one-shots with a persistent note clock: a hit followed by
  `Fm 0` / `Silence` rings out naturally through the silence instead of
  cutting or retriggering.
- A new drum note on the same voice chokes the previous one — drum-machine
  behavior, which also means closed hats choke open hats inside one `Seq`.
- Loudness calibration is enforced by tests
  (`verify_drum_balance`, `verify_preset_balance` in
  `src/testing/loudness_balance_test.rs`). If you change synthesis code,
  rerun `cargo test --release measure_preset_loudness -- --nocapture` for
  fresh `gain_trim` corrections.
- Preset values live in `weresocool_synth/src/presets.rs`; preset *names*
  in `ast/src/drum_presets.rs` (the parser validates against them).

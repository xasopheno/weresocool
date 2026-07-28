# Drums

WereSoCool has ten synthesized drums — `Kick`, `Snare`, `HiHat` (plus
`OpenHat`, the open variant), `Clap`, `Rimshot`, `Tom`, `Ride`, `Crash`,
`Shaker`, and `Cowbell` — built on physical modeling: Karplus-Strong
waveguides, 22-mode Bessel cymbal physics, stateful resonant filters,
velocity-dependent timbre, and analog-style saturation and compression.

Congas and bongos are `Tom` presets; tambourine, maraca and cabasa are
`Shaker` presets — same physics, different shell.

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

### Tom

| Preset | Character |
|---|---|
| `wsc` (default) | The signature tom: waveguide body, wooden shell ring, a stick you can hear |
| `808` | The long tuned boom — sine-dominant, almost no stick |
| `909` | Tighter and drier, more beater |
| `acoustic` | A played drum — shell ring forward, broadband contact noise |
| `conga` | Hand drum: higher, faster, no stick — the bongo/conga voicing |
| `dust` | Lofi — dark, soft, short |
| `glass` | Bright crystalline ring — pitched modes, noise ≈ 0 |
| `doom` | Sub-tuned, very long, dark |
| `crush` | Bit-crushed and driven |
| `air` | Breathy wash, no transient |
| `snap` | Transient only, no tail |

### Ride

| Preset | Character |
|---|---|
| `wsc` (default) | Ping over wash — the stick reads, the body sustains |
| `909` | Bright machine ride |
| `acoustic` | A real ride: defined bell, long complex wash |
| `bell` | Bell-forward — the ping dominates |
| `dark` | Low and washy, ping pulled back |
| `glass` `doom` `crush` `air` `snap` | The character set |

### Crash

| Preset | Character |
|---|---|
| `wsc` (default) | Blooms — peaks ~21 ms in, then a long decay |
| `909` | Bright machine crash |
| `acoustic` | A struck cymbal, wide and long |
| `splash` | Small and fast — up and gone |
| `china` | Trashy, fast attack, ragged spectrum |
| `glass` `doom` `crush` `air` `snap` | The character set |

### Shaker

| Preset | Character |
|---|---|
| `wsc` (default) | Particles — a sample-and-hold grain over a jittered burst train |
| `tambourine` | Jingles ring on after the shake |
| `maraca` | Harder, drier, more shell |
| `cabasa` | Beads over metal — a rougher grain |
| `808` | Machine shaker |
| `glass` `doom` `crush` `air` `snap` | The character set |

### Cowbell

| Preset | Character |
|---|---|
| `wsc` (default) | Two detuned partials with a struck edge |
| `808` | The iconic pair of detuned squares |
| `acoustic` | A real bell — more inharmonic, longer clank |
| `glass` `doom` `crush` `air` `snap` | The character set |

## The register anchor

Cymbals and shakers are unpitched, so reading the played note raw puts them
wherever the header happens to be — which is why the hi-hat needs `Fm 50` in
every kit. The five new drums anchor themselves instead:

```
f_base = base_pitch · tune · (note / 60)^pitch_track
```

`60` is `DRUM_REF_FREQ`, the canonical drum header every kit uses, so a
preset's `base_pitch` reads directly as "what Hz this drum sits at under a
standard header". `pitch_track` decides how much the played note moves it:

| Drum | `pitch_track` | What `Fm` does |
|---|---|---|
| `Tom` | 1.0 | A real interval. `Tom \| Fm 2` is an honest octave, and a three-tom kit is `Fm 2/3` / bare / `Fm 3/2` |
| `Cowbell` | 0.5 | Moves, but half as far as you ask |
| `Ride` `Crash` `Shaker` | 0.25–0.40 | Colours the metal without transposing it off the map |

So the new voices need no `Fm` in a kit unless you want one.

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
| Tom | `attack` (mallet→stick), `body` (thin→thick), `tone` (dark→bright), `length` (tight→booming) |
| Ride | `attack` (soft→pinging), `metal` (dull→shimmery), `length` (short→sustaining) |
| Crash | `attack` (swelling→immediate), `metal` (dull→shimmery), `length` (splash→long) |
| Shaker | `attack` (soft→spitty), `tone` (dark→bright), `length` (tick→long shake) |
| Cowbell | `tone` (dull→clangy), `length` (tick→clank) |

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

**Tom**: `tune`, `pitch_decay` (s), `pitch_range` (start multiple),
`amp_decay` (s), `saturation`, `ks_mix` (waveguide blend), `attack_amount`
(stick), `shell` (wood ring), `velocity_tilt`.

**Ride**: `tune`, `decay_rate` (per second), `shimmer`, `brightness`,
`attack_amount`, `bell_amount` (the ping), `wash` (the body under it),
`velocity_tilt`.

**Crash**: `tune`, `decay_rate` (per second), `shimmer`, `brightness`,
`swell` (how far into the hit it peaks), `wash`, `velocity_tilt`.

**Shaker**: `tune`, `decay` (s), `density` (particles per shake),
`brightness`, `jingle` (rings on after — the tambourine axis),
`velocity_tilt`.

**Cowbell**: `tune`, `decay` (s), `ratio` (detuning between the two
partials), `attack_amount` (the struck edge), `velocity_tilt`.

## Kits

`kits/` contains ready-made kit files (`kit_808`, `kit_909`, `kit_trap`,
`kit_acoustic`, `kit_industrial`, `kit_minimal`, `kit_boom_bap`) — each
defines the full kit with mix staging, ready to copy or adapt:

| | |
|---|---|
| `bd` `sn` `hh` `oh` `cp` `rs` | kick, snare, closed and open hat, clap, rimshot |
| `lt` `mt` `ht` | low / mid / high tom — one voice at three registers |
| `rd` `cr` | ride, crash |
| `sh` `cb` | shaker, cowbell |

`kits/showcase.socool` plays every preset in every family in sequence.

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
- There are **no choke groups across voices**. `Ride` and `Crash` written on
  separate `Overlay` lines ring through each other, as do `OpenHat` and
  `HiHat`. Long cymbals make this much more audible than it used to be.
- Loudness calibration is enforced by tests
  (`verify_drum_balance`, `verify_preset_balance` in
  `src/testing/loudness_balance_test.rs`). If you change synthesis code,
  rerun `cargo test --release measure_preset_loudness -- --nocapture` for
  fresh `gain_trim` corrections.
- Preset values live in `weresocool_synth/src/presets.rs`; preset *names*
  in `ast/src/drum_presets.rs` (the parser validates against them).

use crate::play::play_once;
use crate::Error;
use indoc::indoc;
use weresocool::interpretable::InputType::Language;
use weresocool::manager::prepare_render_outside;

pub fn demo() -> Result<(), Error> {
    // let render_voices = prepare_render_outside(Language(DEMO), None);
    // play_once(render_voices?, "demo.socool".to_string())?;
    todo!();
    Ok(())
}

const DEMO: &str = indoc! {"
{ f: 440, l: 1, g: 1/3, p: 0 }
thing1 = {
  Overlay [
      O[
        (1/1, 2, 1, 2/5),
        (1/1, 0, 1, -2/5),
      ]
      | Seq [
        Fm 1, Fm 0, Fm 0
      ] | Lm 1/3,
      Seq [
          O[
            (1/2, 0.1, 1, 1/8),
            (1/2, 0, 1, -1/8),
          ]
          | Seq [Lm 7, Lm 0] | Lm 1/8
      ]
  ]
  | Seq [
    Fm 1, Fm 3/2, Fm 5/6, Fm 1/2, 
    Fm 5/4, Fm 3/4, Fm 15/8, Fm 1/2, 
    Fm 2, Fm 5/8, Fm 2/3 | Pa 1, Fm 9/4, 
    Fm 5/6, Fm 2/3, Fm 9/16, Fm 1/2,
    Fm 5/2, Fm 11/4, Fm 1/2, Fm 3,
    Fm 25/12, Fm 5/4, Fm 5/3, Fm 25/24,
    Fm 9/8, Fm 8/4, Fm 5/6, Fm 2/3 | Pa -1,
    Fm 3/4 | Pa -1, Fm 4/3, Fm 15/8, Fm 8/3,
    Fm 7/3, Fm 7/8, Fm 3/4, Fm 9/8,
    Fm 5/8 | Pa 1, Fm 11/8, Fm 5/2, Fm 1/2 | Pa -1,
    Fm 5/2 | Pa -1, Fm 2/3, Fm 5/2, Fm 8/3 | Pa 1,
    Fm 9/8, Fm 5/4, Fm 1/2, Fm 8/3 | Pa -1, 
    Fm 3/2, Fm 1, Fm 5/8, Fm 3, 
    Fm 10/3 | Pa -1, Fm 4/3, Fm 1/2, Fm 16/5 | Pa 1
  ]
  | Lm 1/8
  | Repeat 3
}

thing2 = {
  Overlay [
    Seq [
      Fm 0 | Lm 4, Fm 1, Fm 0 | Lm 3
    ],
    Seq [
      Fm 0 | Lm 5, Fm 2, Fm 0 | Lm 3
    ] | Gm 1/8,
    Seq [
      Fm 0 | Lm 6, Fm 3, Fm 0 | Lm 3
    ] | Gm 1/32,
    Seq [
      Fm 0 | Lm 7, Fm 4, Fm 0 | Lm 3
    ] | Gm 1/64,
    Seq [
      Fm 0 | Lm 8, Fm 1/2, Fm 0 | Lm 3
    ] | Gm 1/32
  ]
  | Seq [
      O[
        (3, 0, 1/8, 1/8),
        (5/4, 2, 1, 1/2),
        (1/2, 0, 1, -1/2),
        (1/4, 0, 1, 1/2),
      ],
      O[
        (10/3, 0, 1/8, -1/8),
        (4/3, 2, 1, 1/2),
        (9/16, 0, 1, -1/2),
        (3/16, 0, 1, 1/2),
      ],
      O[
        (15/4, 0, 1/8, -1/8),
        (3/2, 2, 1, 1/2),
        (5/8, 0, 1, -1/2),
        (3/8, 0, 1, 1/2),
      ],
      O[
        (15/4, 0, 1/8, 1/8),
        (3/2, 2, 1, 1/2),
        (3/8, 0, 1, -1/2),
        (3/16, 0, 1, 1/2),
      ],
      O[
        (5/2, 0, 1/8, 1/8),
        (5/3, 2, 1, 1/2),
        (1/3, 0, 1, -1/2),
        (5/12, 0, 1, -1/2),
      ],
      O[
        (8/3, 0, 1/8, -1/8),
        (15/8, 2, 1, 1/2),
        (9/16, 0, 1, -1/2),
        (3/8, 0, 1, -1/2),
      ],
      O[
        (9/4, 0, 1/4, 1/8),
        (2, 2, 1, 1/2),
        (2/3, 0, 1, -1/2),
        (1/3, 0, 1, -1/2),
      ],
      O[
        (5/2, 0, 1, -1/8),
        (2, 2, 1, 1/2),
        (5/8, 0, 1, -1/2),
        (1/2, 0, 1, 0),
        (1/4, 0, 1, 0),
      ] | Lm 2,
  ]
  | Overlay [
    Sine | Fa -3 | Pm -1 | Gm 1/5, 
    Sine | Gm 1/2, 
    Sine 1.6 | Gm 1/8 | Fm 1/2 | Fa -7,
    Pm -1/8 | Sine 1.75 | Gm 1/8 | Fm 1/2,
  ]
  | FitLength thing1
}

bass = {
  O[
    (3/4, 2, 1, 1),
    (3/4, 0, 1, 1),
    (1/1, 1, 1, -1),
    (1/1, 0, 1, -1),
  ]
  | Fa 10
  | Fm 2
  | Gm 1/2
  | Seq [Fm 1, Fm 0, Fm 0] | Repeat 8
  | Seq [
    Fm 1
  ]
  | Seq [Pm 1, Pm -1]
  | Repeat 2
  | Repeat 4 
  | FitLength thing1
}

drums = {
  Noise | 
  O[
    (1/1, 1, 1, 1/6),
    (2, 0, 1, 1/8),
  ]
  | Gm 1/2
  | Seq [Fm 1, Fm 0, Fm 0]
  | Seq [Fm 1, Fm 1/4]
  | Seq [
    Fm 1
  ]
  | Seq [Pm 1, Pm -1]
  | Repeat 6
  | Repeat 8
  | ModBy [
    Gm 0, Gm 1
  ]
  | Gm 1/6
  | FitLength thing1
}


main = {
  Overlay [
    thing1,
    thing2,
    bass,
    drums
  ]
  | Seq[
    Lm 1, 
    Fa 8 | Lm 9/11
  ]
}
"};

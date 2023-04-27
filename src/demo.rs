use crate::Error;
use indoc::indoc;
use std::sync::Arc;
use std::sync::Mutex;
use weresocool::interpretable::InputType::Language;
use weresocool::manager::prepare_render_outside;
use weresocool::manager::RenderManager;
use weresocool::portaudio::real_time_render_manager;
use weresocool::ui::were_so_cool_logo;

pub fn demo() -> Result<(), Error> {
    were_so_cool_logo(Some("Playing"), Some("Demo".to_owned()));

    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    let render_manager = Arc::new(Mutex::new(RenderManager::init(None, Some(tx), true, None)));
    let render_voices = prepare_render_outside(Language(DEMO), None)?;

    render_manager
        .lock()
        .unwrap()
        .push_render(render_voices, true);

    let mut stream = real_time_render_manager(Arc::clone(&render_manager))?;

    stream.start()?;
    // rx.recv blocks until it receives data and
    // after that, the function will complete,
    // stream will be dropped, and the application
    // will exit.
    match rx.recv() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };
    Ok(())
}

const DEMO: &str = indoc! {"
{ f: 311.127, l: 1, g: 1/3, p: 0 }

thing2 = {
    Overlay [
        -- {3/1, 3, 1/16, 1/8},
        -- {3/1, 0, 1/16, -1/8},
        {2/1, 2, 1/2, 1/4},
        {2/1, 0, 1/2, -1/4},
        {1/2, 3, 1, 1},
        {1/2, 0, 1, -1},
    ]
    | Seq [
        Fm 3/4, Fm 0, Fm 9/8, Fm 5/6,
        Fm 3/4, Fm 0, Fm 9/8, Fm 5/6,
        Fm 1, Fm 0, Fm 1, Fm 0,
        Fm 1, Fm 15/16, Fm 5/6, Fm 3/4,
        Overlay [
            Seq [
                Fm 1 | Pa 1/2, Fm 1 | Pa -1/2, Fm 0
            ] 
            | Lm 1/3
            | Seq [
                Fm 2/3, Fm 1 | Gm 1/8, 
                Fm 5/4, Fm 5/6, Gm 1/8, 
                Fm 2/3, Fm 3/4 | Gm 1/8, 
                Fm 15/16, Fm 9/8
            ] | Lm 1/2,
            Seq [
                Fm 2/3
            ] | Lm 4 | Fm 1/4
        ],
        Fm 1, Fm 5/8, Fm 2/3, Fm 5/4,
        Fm 9/8, Fm 3/4, Fm 9/8, Fm 3/4,
        Fm 9/8, Fm 5/4, Fm 3/2 | Lm 2,
        Fm 5/3 | Lm 2, Fm 3/2, Fm 5/3,
        Fm 5/3, Fm 3/2, Fm 4/3, Fm 5/4,
        Fm 4/3, Fm 5/4, Fm 3/2, Fm 9/8,
        Fm 1, Fm 15/16, Fm 5/6, Fm 3/4,
        Fm 5/6, Fm 15/16, Fm 1, Fm 9/8,
        Fm 5/4, Fm 9/8, Fm 5/4, Fm 5/6,
        Fm 3/4, Fm 5/6, Fm 3/4, Fm 1/2,
        Seq [
            Fm 1/2, Fm 3/4, Fm 5/6, Fm 1/2, Fm 1/6
        ] 
        | Seq [
          Repeat 7, 
          Fm 7/8 | Reverse
        ]
        | Repeat 4
    ]
}


thing3 = {
    Overlay [
        -- {3, 2, 1/16, 1/7},
        -- {3, 0, 1/16, -1/7},
        {1/1, 1, 1/2, 1/4},
        {1/1, 0, 1/2, -1/4},
    ]
    | Seq [
        Fm 1/2, Fm 0, Fm 3/4, Fm 1/2,
        Fm 9/16, Fm 0, Fm 3/4, Fm 2/3,
        Fm 2/3, Fm 0, Fm 5/8, Fm 0,
        Fm 5/4, Fm 9/8, Fm 1, Fm 15/16,

        Seq [
            Fm 1 | Pa 1/2, Fm 1 | Pa -1/2, Fm 0
        ] 
        | Lm 1/3
        | Seq [
            Fm 5/6, Fm 5/4 | Gm 1/8, 
            Fm 3/2, Fm 1 | Gm 1/8, 
            Fm 5/6, Fm 15/16, Gm 1/8, 
            Fm 9/8, Fm 4/3
        ] | Lm 1/2,
        Fm 5/4, Fm 3/4, Fm 5/6, Fm 1,
        Fm 15/16, Fm 5/8, Fm 15/16, Fm 5/8,
        Fm 15/16, Fm 1, Fm 5/4 | Lm 2,
        Fm 4/3 | Lm 2, Fm 5/4, Fm 4/3,
        Fm 4/3, Fm 5/4, Fm 9/8, Fm 5/4,
        Fm 9/8, Fm 1, Fm 5/4, Fm 15/16,
        Fm 5/6, Fm 3/4, Fm 2/3, Fm 5/8,
        Fm 2/3, Fm 3/4, Fm 3/4, Fm 5/8,
        Fm 1, Fm 15/16, Fm 1, Fm 2/3,
        Fm 5/8, Fm 2/3, Fm 5/8, Fm 1/2,
        Seq [
            Fm 1/2, Fm 3/4, Fm 5/6, Fm 1/2, Fm 1/6
        ] 
        | Seq [Repeat 7, Fm 7/8 | Reverse]
        -- | Repeat 3
    ]
}

chord = {
    Seq [
        Fm 0,
        Overlay [
            {2/1, 1, 1/8, 1/2},
            {2/1, 0, 1/8, -1/2},
            {5/3, 2, 1/2, 1/2},
            {5/3, 0, 1/2, -1/2},
            {9/8, 1, 1/8, 1/2},
            {9/8, 0, 1/8, -1/2},
        ],
        Overlay [
            {15/8, 1, 1/8, 1/2},
            {15/8, 0, 1/8, -1/2},
            {3/2, 5, 1/2, 1/2},
            {3/2, 0, 1/2, -1/2},
            {5/4, 1, 1/8, 1/2},
            {5/4, 0, 1/8, -1/2},
        ],
        Overlay [
            {3/2, 1, 1/8, 1/2},
            {3/2, 0, 1/8, -1/2},
            {5/4, 6, 1/2, 1/2},
            {5/4, 0, 1/2, -1/2},
            {9/8, 1, 1/2, 1/2},
            {9/8, 0, 1/2, -1/2},
        ] | Seq [Fm 1 | Lm 2, Fm 0 | Lm 6, Fm 0 | Lm 2] | Lm 1/4,
        Fm 0 | Repeat 6,
    ]
    | FitLength thing2
}

chord2 = {
  Seq [
    Seq [Fm 1, Fm 0, Fm 0] | Lm 1/3
    | Seq [
        Fm 0 | Lm 9,
        Overlay [
            {3, 5, 1/64, 1/8},
            {3, 0, 1/64, -1/8},
            {5/2, 5, 1/16, 1/8},
            {5/2, 0, 1/16, -1/8},
            {5/3, 4, 1/8, 1/2},
            {5/3, 5, 1/8, -1/2},
            {3/2, 1, 1/8, 1/2},
            {3/2, 0, 1/8, -1/2},
        ] | Lm 3,
        Overlay [
            {11/4, 6, 1/64, 1/8},
            {11/4, 0, 1/64, -1/8},
            {5/2, 6, 1/16, 1/8},
            {5/2, 0, 1/16, -1/8},
            {7/4, -3, 1/8, 1/2},
            {7/4, 0, 1/8, -1/2},
            {5/3, 2, 1/8, 1/2},
            {5/3, 0, 1/8, -1/2},
            {3/2, -4, 1/8, 1/2},
            {3/2, 0, 1/8, -1/2},
        ],
        Overlay [
            {15/4, 7, 1/64, 1/2},
            {15/4, 0, 1/64, -1/2},
            {20/3, 7, 1/16, 1/2},
            {20/3, 0, 1/16, -1/2},
            {9/4, -6, 1/8, 1/2},
            {9/4, 0, 1/8, -1/2},
            {7/4, -2, 1/8, 1/2},
            {7/4, 0, 1/8, -1/2},
            {8/5, 4, 1/8, 1/2},
            {8/5, 0, 1/8, -1/2},
            {3/2, -4, 1/8, 1/2},
            {3/2, 0, 1/8, -1/2},
            {5/8, -4, 1/8, 1/2},
            {5/8, 0, 1/8, -1/2},
        ],
        Overlay [
            {15/4, 8, 1/64, 1/2},
            {15/4, 0, 1/64, -1/2},
            {20/3, 8, 1/16, 1/2},
            {20/3, 0, 1/16, -1/2},
            {11/4, -7, 1/8, 1/2},
            {11/4, 0, 1/8, -1/2},
            {7/4, -3, 1/8, 1/2},
            {7/4, 0, 1/8, -1/2},
            {8/5, 5, 1/8, 1/2},
            {8/5, 0, 1/8, -1/2},
            {3/2, -3, 1/8, 1/2},
            {3/2, 0, 1/8, -1/2},
            {5/8, -4, 1/8, 1/2},
            {5/8, 0, 1/8, -1/2},
        ],
    ],
    Overlay [
        {4, -7, 1/8, 1/2},
        {4, 0, 1/8, -1/2},
        {15/4, 1, 1/64, 1/2},
        {15/4, 0, 1/64, -1/2},
        {3, -7, 1/8, 1/2},
        {3, 0, 1/8, -1/2},
        {5/6, -4, 1/8, 1/2},
        {5/6, 0, 1/8, -1/2},
        {3/2, 5, 1/8, 1/2},
        {3/2, 0, 1/8, -1/2},
        {4/3, -3, 1/8, 1/2},
        {4/3, 0, 1/8, -1/2},
        {1/2, -4, 1/8, 1/2},
        {1/2, 0, 1/8, -1/2},
    ],
    Overlay [
        {4, -7, 1/8, 1/2},
        {4, 0, 1/8, -1/2},
        {10/3, 1, 1/64, 1/2},
        {10/3, 0, 1/64, -1/2},
        {3, -9, 1/8, 1/2},
        {3, 0, 1/8, -1/2},
        {15/8, -5, 1/8, 1/2},
        {15/8, 0, 1/8, -1/2},
        {3/2, 5, 1/8, 1/2},
        {3/2, 0, 1/8, -1/2},
        {5/4, -3, 1/8, 1/2},
        {5/4, 0, 1/8, -1/2},
        {1/2, -4, 1/8, 1/2},
        {1/2, 0, 1/8, -1/2},
    ]
    | Seq [
        Fm 1 | Lm 4, Fm 3/4, Fm 1/2, Fm 1/3, Fm 1/4, Fm 1/5, Fm 1/6, Fm 1/7, Fm 1/8,
      ] | Lm 1/16,
    ]
    | Gm 7/4
    | Overlay [Sine, Sine 3/2 | Gm 1/8, Sine 2 | Gm 1/8]
    | Overlay [AsIs, Reverb 1]
    | FitLength thing2
}

section1 = {
    Overlay [
        thing2,
        thing3,
        chord,
        chord2,
    ]
    | Fm 9/8
    | Lm 1/7
    | Repeat 1
}

new_thing = {
  Seq [
    Overlay [
        {15/8, -5, 1, 1/2},
        {15/8, 0, 1, -1/2},
        {3/2, 5, 1, 1/2},
        {3/2, 0, 1, -1/2},
        {5/4, -3, 1, 1/2},
        {5/4, 0, 1, -1/2},
        {5/6, -4, 1, 1/2},
        {5/6, 0, 1, -1/2},
    ],
    Overlay [
        {15/8, -5, 1, 1/2},
        {15/8, 0, 1, -1/2},
        {5/3, 5, 1, 1/2},
        {5/3, 0, 1, -1/2},
        {9/8, -3, 1, 1/2},
        {9/8, 0, 1, -1/2},
        {3/4, -4, 1, 1/2},
        {3/4, 0, 1, -1/2},
    ],
  ] | Lm 4
}

new_thing_bass = {
    Overlay [
        {9, -8, 1/48, 1/4},
        {9, 0, 1/48, -1/4},
        {4, -2, 1/4, 1},
        {4, 0, 1/4, -1},
        {2, -2, 1, 1/7},
        {2, 0, 1, -1/7},
        {3/2, -3, 1, 1/7},
        {3/2, 0, 1, -1/7},
        {1/2, -3, 1, 1/7},
        {1/2, 0, 1, -1/7},
    ]
    | Seq [
      Fm 0, Fm 5/6 | Lm 4/5, Fm 3/4 | Lm 6/5, Fm 5/8,
      Fm 5/6, Fm 15/16, Fm 3/4 | Lm 4/5, Fm 5/6 | Lm 6/5,
      Seq [
        Fm 3/4, Fm 5/6, 
        Overlay [
          Fm 9/8 | Gm 1/2,
          Fm 9/16,
          Fm 5/3 | Pa 1 | Gm 1/8,
          Fm 5/3 | Fa 3 | Pa -1 | Gm 1/8,
        ], Fm 0
      ] 
      | Lm 2/3
    ]
}

new_chord = {
  Overlay [
    {11/4, -7, 1/2, 1},
    {11/4, 0, 1/2, -1},
    {5/2, -7, 1/2, 1/3},
    {5/2, 0, 1/2, -1/3},
    {9/4, -3, 1/2, 1/4},
    {9/4, 0, 1/2, -1/4},
    {15/8, 8, 1/2, 1/4},
    {15/8, 0, 1/2, -1/4},
    {3/2, -3, 1/2, 1},
    {3/2, -7, 1/2, -1},
    {1/5, -3, 1/2, 1},
    {1/5, -7, 1/2, -1},
  ]
  | Gm 1/2
  | FitLength new_thing_bass
}

section2 = {
  Overlay [
    {1, 1/7, 1/2, 1},
    {1, 0, 1, 0},
    {1, 1/10, 1/2, -1},
  ]
  | Overlay [
    new_thing,
    new_thing_bass,
    new_chord
  ] 
  | Gm 1/3
  | Seq [
    Seq [
        Lm 5/4, 
        Lm 5/4 | ModBy [Fm 1 | Lm 25/2, Fm 1 | ModBy [Fm 2/3, Fm 1/2, Fm 1/3, Fm 0 | Lm 1/3]],
        Lm 1,
        Lm 1 | ModBy [Fm 1 | Lm 19/2, Fm 1 | ModBy [Fm 2/3, Fm 1/2, Fm 1/3]],
    ]
  ]
}

main = {
  Seq [
    section1 | Repeat 2,
    Fm 0 | Lm 1/100,
    section2 | Lm 3/4,
    Fm 0 | Lm 1/100,
  ]
  | Gm 1
  | Repeat 2

}
"};

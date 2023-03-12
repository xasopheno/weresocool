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

chord = {
    Overlay [
        {9/2, 0, 1/2, -1},
        {9/2, 0, 1/2, -1},
        {4, 0, 1/2, -1},
        {4, 0, 1/2, -1},
        {10/3, 0, 1/2, -1},
        {16/5, 0, 1/2, -1},
        {5/4, 4, 1/2, 1},
        {8/3, 0, 1/2, -1},
        {5/4, 4, 1/2, 1},
        {7/3, 0, 1/2, -1},
        {9/4, 4, 1/2, 1},
        {3/2, 4, 1/2, 1},
        {11/8, 0, 1/2, -1},
        {5/4, 7, 1/2, 1},
        {7/6, 0, 1/2, -1},
        {3/2, 4, 1, 1/4},
        {3/2, 0, 1, -1/4},
        {1/3, 4, 1, 1/4},
        {1/3, 0, 1, -1/4},
    ]
    | Fm 9/8 
}

overtones1 = {
    Overlay [
        {1/1, 2, 1, 1/2},
        {1/1, 0, 1, -1/2},
        {3/4, 3, 1/8, 1},
        {3/4, 0, 1/8, -1},
        {0/1, 7, 1, 1},
        {0/1, 0, 1, -1},
    ]
}

overtones2 = {
    Overlay [
        {4, -9, 1/7, 1/4},
        {4, 0, 1/7, -1/4},
        {1/2, 2, 1, 1},
        {1/2, 0, 1, -1},
        {1/4, 2, 1, 1},
        {1/4, 0, 1, -1},
    ]
}

bass = {
    Seq [
        Fm 7/8, Fm 1, Fm 3/2, Fm 9/8, Fm 5/4, Fm 4/3 | Lm 2, Fm 5/4, Fm 7/8,
        Fm 1, Fm 5/4, Fm 4/3, Fm 1, Fm 5/4, Fm 5/3, Fm 9/8, Fm 5/6, Fm 7/8,
   ]
   | Overlay [Sine, Sine 3/2 | Fm 1/2 | Gm 1/7]
   | Seq [Repeat 2, Fm 5/6, Reverse]
}

thing2 = {
    Overlay [
        {2, 7, 1/2, 1/5},
        {2, 0, 1/2, -1/5},
        {3/2, 4, 1, 1/4},
        {3/2, 0, 1, -1/4},
        {1/1, -7, 1, 1/7},
        {1/1, 0, 1, -1/7},
    ]
    | Overlay [
        Square | Gm 1/7,
        Sine | Gm 1/7 | Pm 2,
    ]
    | Seq [
        Fm 3/4, Fm 5/6, Fm 7/8, Fm 9/8, Fm 1, Fm 7/8, Fm 9/8, Fm 1, Fm 5/6
    ]
    | Overlay [
        {1/1, -3, 1, 1},
        {1/1, 0, 1, -1},
    ]
    | Gm 1/2
    | Repeat 2
    | FitLength bass
}

thing6 = {
    Overlay [
        {1/1, 3, 1/2, -1},
        {1/1, -2, 1/2, 1},
    ] 
    | Seq [
        Fm 2, Fm 2, Fm 2, Fm 2, Fm 5/2, Fm 5/2, Fm 9/4, Fm 9/4,
        Fm 3, Fm 2, Fm 2, Fm 9/4, Fm 5/2, Fm 5/2, Fm 9/4, Fm 9/4,
    ]
    | Lm 2
}

thing3 = {
    Overlay [
        {1/1, 5, 1/2, -1},
        {1/1, -2, 1/2, 1},
    ] 
    | Seq [
        Fm 3/2, Fm 4/3, Fm 5/3, Fm 3/2, Fm 2, Fm 3/2, Fm 5/3, Fm 15/8,
        Fm 1, Fm 1, Fm 4/3, Fm 5/3, Fm 4/3, Fm 3/2, Fm 5/3, Fm 15/8,
    ]
    | Lm 2
}

thing4 = {
    Overlay [
        {1/1, 2, 1/2, -1},
        {1/1, 0, 1/2, 1},
        {1/2, 4, 1/2, 1},
        {1/2, 0, 1/2, -1},
    ] 
    | Seq [
        Fm 5/4, Fm 9/8, Fm 4/3, Fm 5/4, Fm 5/3, Fm 5/4, Fm 4/3, Fm 3/2,
        Fm 1, Fm 5/6, Fm 9/8, Fm 4/3, Fm 9/8, Fm 5/4, Fm 4/3, Fm 3/2,
    ]
    | Lm 2
}

thing5 = {
    Overlay [
        {1/2, -1, 1/2, 1},
        {1/2, 0, 1/2, -1},
        {1/4, -1, 1/2, 1},
        {1/4, 0, 1/2, -1},
    ] 
    | Seq [
        Fm 1, Fm 1, Fm 1, Fm 3/4, Fm 2/3, Fm 5/8, Fm 9/16, Fm 9/16,
        Fm 2/3, Fm 2/3, Fm 3/4, Fm 3/4, Fm 9/16, Fm 5/8, Fm 2/3, Fm 3/4,
    ]
    | Lm 2
}

main = {
    Seq [
        Seq [
            Fm 25/24 | chord | Lm 1/4 | ModBy [Fm 1, Fm 3/4, Fm 1/2, Fm 1/3, Fm 1/4] | Gm 1/3,
            Fm 0 | Lm 1/100,
            Overlay [
                Overlay [
                    Seq [
                        Fm 0, Fm 1 
                    ] | Lm 1/2,
                    Seq [
                        Fm 1, Fm 0 
                    ] | Lm 1/3
                ]
                | overtones1 | bass,
                thing2
            ] 
            | Fm 25/24 
            | Lm 1/7,
            Fm 0 | Lm 1/100,
            chord | Lm 1/4 | ModBy [Fm 1, Fm 3/4, Fm 1/2, Fm 1/3, Fm 1/4] | Gm 1/3,
            Fm 0 | Lm 1/8,
            Overlay [
                overtones2 | bass,
                thing2
            ] 
            | Repeat 2
            | Lm 1/5,
            Fm 0 | Lm 1/100,
        ]
        | Repeat 2,
        Seq [
            Overlay [
                thing6,
                thing3,
                thing4,
                thing5,
            ]
            | Repeat 2
            | Overlay [
                {1/1, -0.07, 1/3, -1},
                {1/1, 0, 1/2, 0},
                {1/1, 0.03, 1/3, 1},
            ],
        ],
        Fm 0 | Lm 1/100,
        Fm 25/24 | chord | Lm 1/4 | ModBy [Fm 1, Fm 3/4, Fm 1/2, Fm 1/3, Fm 1/4] | Gm 1/3,
        Fm 0 | Lm 1/100,
        Overlay [
            Overlay [
                Seq [
                    Fm 0, Fm 1 
                ] | Lm 1/2,
                Seq [
                    Fm 1, Fm 0 
                ] | Lm 1/3
            ]
            | overtones1 | bass,
            thing2
        ] 
        | Fm 25/24 
        | Lm 1/7,
        Fm 0 | Lm 1/100,
        chord | Lm 1/4 | ModBy [Fm 1, Fm 3/4, Fm 1/2, Fm 1/3, Fm 1/4] | Gm 1/3,
    ]
}
"};

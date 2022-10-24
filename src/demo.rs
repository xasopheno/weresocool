use crate::play::play_once;
use crate::Error;
use indoc::indoc;
use weresocool::interpretable::InputType::Language;
use weresocool::manager::prepare_render_outside;

pub fn demo() -> Result<(), Error> {
    let render_voices = prepare_render_outside(Language(DEMO), None);
    play_once(render_voices?)?;
    Ok(())
}

const DEMO: &str = indoc! {"
{ f: 311.127, l: 1, g: 1, p: 0 }

thing1 = {
  O[
    (1/1, 2, 1, 1),
    (1/1, 0, 1, -1),
  ]
  | Seq [
    Fm 1, Fm 9/8, Fm 5/4
  ]
}

thing2 = {
  O[
    (1/1, 2, 1, 1),
    (1/1, 0, 1, -1),
  ]
  | Seq [
    Fm 3/4
  ]
  | FitLength thing1
}

main = {
  Overlay [
    thing1,
    thing2
  ]
}
"};

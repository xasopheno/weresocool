extern crate weresocool;
use weresocool::compositions::sound_effects::events;
use weresocool::write::write_composition_to_json;

fn main() {
    println!("{}", "\n  ****** WereSoCool __!Now In Stereo!__ ****** ");
    println!("{}", "*** Make cool sounds. Impress your friends ***  \n");
    println!("{}", "  )))***=== Composition -> JSON ===***(((  \n ");

    let file_name = String::from("composition.json");
    let written = write_composition_to_json(events, &file_name);

    match written {
        Ok(()) => {},
        _ => {}
    }

    println!("{}", "\n ***** WereSoFinishedWritingTheJSONFile ****** \n ");
}

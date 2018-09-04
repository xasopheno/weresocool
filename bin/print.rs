extern crate weresocool;
use weresocool::compositions::song_15::generate_composition;
use weresocool::write::write_composition_to_wav;

fn main() {
    println!("{}", "\n  ****** WereSoCool __!Now In Stereo!__ ****** ");
    println!("{}", "*** Make cool sounds. Impress your friends ***  ");
    println!("{}", "       )))***=== PRINTER ===***(((  \n ");

    write_composition_to_wav(generate_composition);

    println!("{}", "\n ***** WereSoFinishedWritingTheWavFile ****** \n ");
}

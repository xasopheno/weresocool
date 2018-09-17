extern crate weresocool;
use weresocool::compositions::song_18::generate_composition;
use weresocool::write::write_composition_to_wav;

fn main() {
    println!("{}", "\n  ****** WereSoCool __!Now In Stereo!__ ****** ");
    println!("{}", "*** Make cool sounds. Impress your friends ***  \n");
    println!("{}", "  )))***=== Composition -> WAV ===***(((  \n ");

    write_composition_to_wav(generate_composition);

    println!("{}", "\n ***** WereSoFinishedWritingTheWavFile ****** \n ");
}

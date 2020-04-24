fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rustc-link-lib=static=portaudio");
    println!("cargo:rustc-link-lib=framework=CoreAudio");
    println!("cargo:rustc-link-lib=framework=AudioUnit");
    println!("cargo:rustc-link-lib=framework=Carbon");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");
    //println!("cargo:rustc-link-search=[static=]PATH");
}


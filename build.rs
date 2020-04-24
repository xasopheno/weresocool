fn main() {
    // For building portaudio statically
    println!("cargo:rustc-link-lib=static=portaudio");
    println!("cargo:rustc-link-lib=framework=CoreAudio");
    println!("cargo:rustc-link-lib=framework=AudioUnit");
    println!("cargo:rustc-link-lib=framework=Carbon");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");
}


use std::env;

fn main() {
    let static_build = match env::var("PORTAUDIO_ONLY_STATIC") {
        Ok(_e) => true,
        _ => false,
    };

    if cfg!(target_os = "macos") && static_build {
        println!("cargo:rustc-link-lib=static=portaudio");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
        println!("cargo:rustc-link-lib=framework=Carbon");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
    }
}

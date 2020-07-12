use std::env;

fn main() {
    let static_build = match env::var("PORTAUDIO_ONLY_STATIC") {
        Ok(_e) => true,
        _ => false,
    };

    if env::var("LAME_STATIC").is_ok() {
        println!("cargo:rustc-link-lib=static=mp3lame");
    }

    println!("cargo:rerun-if-env-changed=PORTAUDIO_ONLY_STATIC");
    if cfg!(target_os = "macos") && static_build {
        println!("cargo:rustc-link-lib=static=portaudio");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
        println!("cargo:rustc-link-lib=framework=Carbon");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
    }
}

use std::env;

fn main() {
    let static_build = matches!(env::var("PORTAUDIO_ONLY_STATIC"), Ok(_e));

    println!("cargo:rerun-if-env-changed=PORTAUDIO_ONLY_STATIC");
    if cfg!(target_os = "macos") && static_build {
        println!("cargo:rustc-link-lib=static=portaudio");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
        println!("cargo:rustc-link-lib=framework=Carbon");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
    }
}

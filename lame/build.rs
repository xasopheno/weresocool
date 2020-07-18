use std::env;

fn main() {
    let static_build = match env::var("PORTAUDIO_ONLY_STATIC") {
        Ok(_e) => true,
        _ => false,
    };

    println!("cargo:rerun-if-env-changed=PORTAUDIO_ONLY_STATIC");
    if cfg!(target_os = "macos") && static_build {
        println!("cargo:rustc-link-lib=static=mp3lame");
    }
}

# ***** WereSoCool __!Now In Stereo!__ ******

Make cool sounds. Impress your friends. 

binaural, microtonal sonification 

## Install
You'll need Rust and Cargo.
`https://www.rust-lang.org/en-US/install.html` 

You'll need also need portaudio. 

On Mac

`brew install portaudio`

## Run
### Composer
Listen to something created with the framework

`cargo run --bin composer --release`

### Print
Print something created with the framework to .wav

`cargo run --bin print --release`

I use ffmpeg to convert the wave file to mp3'

`ffmpeg -i composition.wav composition.mp3`

https://www.ffmpeg.org/

## Test
`cargo test`

# ***** WereSoCool __!Now In Stereo!__ ******

Make cool sounds. Impress your friends. 

binaural, microtonal sound generation

## Install
You'll need portaudio. 
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

### Microphone
Realtime pitch detection from microphone input. The predicted frequencies are used to generate accompanying frequencies.

I recommend you change the buffer_size in settings.rs to ~256. 

`cargo run --bin mic --release`

## Test
`cargo test`

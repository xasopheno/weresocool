# ***** WereSoCool __!Now In Stereo!__ ******

Make cool sounds. Impress your friends. 

binaural, microtonal sonification 

Copyright (C) 2018 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).

## Install
You'll need Rust and Cargo.
`https://www.rust-lang.org/en-US/install.html` 

You'll need also need portaudio. 

On Mac

`brew install portaudio`

## Parser
Currently, the parser needs to be in a directory at the same level as this repository.
It's linked locally through the `Cargo.toml`. 

`https://github.com/xasopheno/weresocool-parser`

Grammar:

https://github.com/xasopheno/weresocool-parser/blob/master/src/socool.lalrpop

## Run
Listen to something created with the framework

`cargo run --release --bin wsc songs/working.socool`

`ffmpeg -i composition.wav composition.mp3`

https://www.ffmpeg.org/

## Building a binary
To build the binary:

`cargo build --release --bin wsc`

and then you can parse and play files without having to build the binary each time.

`./target/release/wsc songs/sweep.socool -p`

## Usage

```
USAGE:
    wsc [FLAGS] [filename]

FLAGS:
    -d, --doc        Prints some documentation
    -h, --help       Prints help information
    -j, --json       Prints file to .json
    -p, --print      Prints file to .wav
    -V, --version    Prints version information

ARGS:
    <filename>    filename eg: my_song.socool
```

## Test
`cargo test`

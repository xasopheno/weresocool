# ***** WereSoCool __!Now In Stereo!__ ******
[![Build Status](https://travis-ci.org/xasopheno/WereSoCool.svg?branch=master)](https://travis-ci.org/xasopheno/WereSoCool)

A language for binaural, microtonal composition built in Rust.

Make cool sounds. Impress your friends/pets/plants.

## Install
You'll need Rust and Cargo.
`https://www.rust-lang.org/en-US/install.html` 

You'll need also need portaudio. 
https://github.com/RustAudio/rust-portaudio

On Mac
`brew install portaudio`
`brew install pkg-config`
`&& cargo clean` if you are having problems linking

## Parser
`https://github.com/xasopheno/weresocool-parser`

Grammar:

https://github.com/xasopheno/weresocool-parser/blob/master/src/socool.lalrpop

## Run
Listen to something created with the framework

`cargo run --release --bin wsc songs/fall/table.socool`


Run with `-p` flag to print a wav file.

`cargo run --release --bin wsc songs/fall/table.socool -p`

I use `ffmpeg` to convert to `mp3`

`ffmpeg -i composition.wav composition.mp3`

https://www.ffmpeg.org/

For files in `songs/*` of type `.socool`
`./play dir/filename`
`./print dir/filename`

## Building a binary
To build the binary:

`cargo build --release --bin wsc`

and then you can parse and play files without having to build the binary each time.

`./target/release/wsc songs/fall/table.socool -p`

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
To run tests for WSC, Parser, AST, and the end-to-end tests run:

`./test_all.sh`

Copyright (C) 2019 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).

![WereSoCool](https://raw.githubusercontent.com/xasopheno/weresocool/master/cover.png)

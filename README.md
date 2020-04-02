# ***** WereSoCool __!Now In Stereo!__ ******
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for binaural, microtonal composition built in Rust.

Make cool sounds. Impress your friends/pets/plants.

Working on additional documentation. :) 

## See some things generated with the language here

https://www.weresocool.org/

## Install
You'll need Rust and Cargo.
`https://www.rust-lang.org/en-US/install.html` 

You'll need also need portaudio. 
https://github.com/RustAudio/rust-portaudio

On Mac
`brew install portaudio`
`brew install pkg-config`
`&& cargo clean` if you are having problems linking

## Grammar:

https://github.com/xasopheno/weresocool-parser/blob/master/src/socool.lalrpop

## Run
Listen to something created with the framework

`cargo run --release --bin wsc songs/fall/table.socool`

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

`./test.sh`

Copyright (C) 2020 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).

![WereSoCool](https://raw.githubusercontent.com/xasopheno/weresocool/master/cover.png)

# ***** WereSoCool __!Now In Stereo!__ ******
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for binaural, microtonal composition built in Rust.

Make cool sounds. Impress your friends/pets/plants.

![WereSoCool](https://raw.githubusercontent.com/xasopheno/weresocool/master/imgs/application.png)

WereSoCool is a programming language for composing microtonal music geometrically. This language doesn't assume
familiarity with either microtonal music or computer programming, but experience with either will certainly help. I
recommend starting with the tutorials and when you feel sufficiently confident, just try making some cool things.
There's no better way to learn than to make stuff. 

## Listen:

Watch/Listen to some examples form the langauage [here](https://www.weresocool.org/play/arcs).

## Make Cool Sounds:

### Macos:

The most recent version of the Macos application can be downloaded [here](https://www.weresocool.org/downloads).
Inside, you'll find a lot of cool tutorials and demos that should help you get started. I recommend starting with the
cool tutorials and doing them in order. If you get stuck or want to share some new sounds you've made, reach out to me weresocool at xasopheno dot com. 

### Linux:
Currently on linux, you'll need to compile this locally. See Development. 


### Windows
This does not currently work on Windows...<sad panda>. If you're interested in using this software on a Windows machine, please
    reach out and I'll work on it. 


## Development:

### Setup
You'll need Rust. Rust is a great language. Install it with [Rustup](https://www.rust-lang.org/en-US/install.html).

You'll need also need [portaudio](https://github.com/RustAudio/rust-portaudio) and [lame](https://lame.sourceforge.io/)

#### Macos:
`brew install portaudio pkg-config lame`

#### Arch:
`sudo pacman -S portaudio pkg-config lame`

#### Ubuntu:
`sudo apt-get portaudio pkg-config lame`

### Build
`make package`

## Run: 
#### Linux
`make run`

#### Macos
`make run_osx`

## Run with Dev Server
`make dev`

## Run Tests:
`make test`

![WereSoCool](https://raw.githubusercontent.com/xasopheno/weresocool/master/imgs/cover.png)

Copyright (C) 2020 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).


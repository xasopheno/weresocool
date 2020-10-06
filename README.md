# ***** WereSoCool __!Now In Stereo!__ ******
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for binaural, microtonal composition built in Rust.

<em>Make cool sounds. Impress your friends/pets/plants.</em>

![WereSoCool](../master/imgs/application.png)
**WereSoCool** is a programming language for composing microtonal music geometrically. This language doesn't assume familiarity with either microtonal music or computer programming, but experience with either will certainly help. I recommend starting with the tutorials and when you feel sufficiently confident,  try to make some cool things. There's no better way to learn than to make stuff. If you get stuck, feel free to reach out to me weresocool at xasopheno dot com. 

## Listen:

Watch/Listen to some examples from the language [here](https://www.weresocool.org/play/arcs).

## Make Cool Sounds:
### Macos:
The most recent version of the Macos application can be downloaded [here](https://www.weresocool.org/downloads).

### Linux:
Currently on linux, you'll need to compile this locally. See Development. 


### Windows
This does not currently work on Windows...<em>sad panda</em>. If you're interested in using this software on a Windows machine, reach out and I'll work on it. 


## Development:

### Setup
You'll need Rust. Rust is a great language. Install it with [Rustup](https://www.rust-lang.org/en-US/install.html).

You'll need also need [portaudio](https://github.com/RustAudio/rust-portaudio) and [lame](https://lame.sourceforge.io/)

#### Macos:
`brew install portaudio pkg-config lame`

#### Linux:
##### Arch:
`sudo pacman -S portaudio pkg-config lame`

##### Ubuntu:
`sudo apt-get portaudio pkg-config lame libmp3lame-dev rpm libasound2-dev`

### Build
`make package`

## Run: 
#### Linux
`make run`

#### Macos
`make run_osx`

#### Run with Dev Server
`make dev`

## Run Tests:
`make test`

## Special Thanks:
This wouldn't exist in a million years if it wasn't for Antonis Stampoulis'
help with language design or the help of friends/programmers like
Sonke Hahn, Hao Lian, Catharine M, Matt Handler, Lee Pender, Amanda Doucette, Khaled Alquaddoomi, 
Alex Kestner, everyone else that has sat down to program with me, and Originate. 
Of course, special thanks to Maria for always listening to my new_weird_sounds 
and programming problems. - Danny

![WereSoCool](../master/imgs/cover.png)

Copyright (C) 2020 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).


# ***** WereSoCool __!Now In Stereo!__ ******
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for binaural, microtonal composition built in Rust.

<em>Make cool sounds. Impress your friends/pets/plants.</em>

![WereSoCool](../main/imgs/application.png)
**WereSoCool** is a programming language for composing microtonal music geometrically. This language doesn't assume familiarity with either microtonal music or computer programming, but experience with either will certainly help. 

## Listen:
Watch/Listen to some examples from the language [here](https://www.weresocool.org/play/arcs).

## Installation:
### Macos:

`brew install macos`

### Arch Linux:

Availabe on the AUR here.

### Cargo:

WereSoCool can be installed via on any unix system via cargo

`cargo install weresocool`

### Windows
This does not currently work on Windows...<em>sad panda</em>. If you're interested in using this software on a Windows machine, reach out and I'll work on it. 


## Development:

### Setup
You'll need Rust. Rust is a great language. Install it with [Rustup](https://www.rust-lang.org/en-US/install.html).

You'll need also need to install the following packages:

#### Macos:
`brew install portaudio pkg-config lame libvorbis`

##### Arch Linux:
`sudo pacman -S portaudio pkg-config lame vorbis-tools`

##### Ubuntu:
`sudo apt-get portaudio pkg-config lame libmp3lame-dev rpm libasound2-dev vorbis-tools`

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
Sönke Hahn, Hao Lian, Catharine M, Matt Handler, Lee Pender, Amanda Doucette, Khaled Alquaddoomi,
Alex Kestner, everyone else that has sat down to program with me.
Of course, special thanks to Maria for always listening to my new_weird_sounds
and programming problems. - Danny

![WereSoCool](../main/imgs/cover.png)

Copyright (C) 2022 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).


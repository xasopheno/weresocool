# ***** WereSoCool __!Now In Stereo!__ ******
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for binaural, microtonal composition built in Rust.

<em>Make cool sounds. Impress your friends/pets/plants.</em>

**WereSoCool** is a programming language for composing microtonal music geometrically. This language doesn't require familiarity with either microtonal music or computer programming, but experience with either will certainly help. 

## Installation:
### Macos:

`brew install weresocool`

### Arch Linux:

Available on the AUR [here](https://aur.archlinux.org/packages/weresocool).

### Cargo:

WereSoCool can be installed via on any unix system via cargo

`cargo install weresocool`

### Windows
This does not currently work on Windows 😔. If you're interested in using this software on a Windows machine, reach out and I'll work on it. 

### WereSoCool CLI
```
USAGE:
    weresocool [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    new      new [filename.socool]
    play     play [filename.socool]
    watch    watch [filename.socool]
    demo     hear a cool sound
    print    print [filename.socool] [flags]
        -a, --all          print all file types
            --csv          print csv file
            --json         print csv file
            --mp3          print mp3 file
            --oggvorbis    print oggvorbis file
        -s, --sound        print sound file
            --stems        print stems as a zip file
            --wav          print wav file
    help     help of the given subcommand(s)
```

## Development:

### Setup
Rust: Rust is a great language. Install it with [Rustup](https://www.rust-lang.org/en-US/install.html).

Just: Commands are issued via [Just](https://github.com/casey/just).

You'll need also need to install the following packages:

#### Macos:
`brew install portaudio pkg-config lame libvorbis`

##### Arch Linux:
`sudo pacman -S portaudio pkg-config lame vorbis-tools`

##### Ubuntu:
`sudo apt-get portaudio pkg-config lame libmp3lame-dev rpm libasound2-dev vorbis-tools`

### Build
`just build`

## Run Tests:
`just test`

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


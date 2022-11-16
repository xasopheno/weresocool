# ***** WereSoCool __!Now In Stereo!__ ******
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for binaural, microtonal composition built in Rust.

<em>Make cool sounds. Impress your friends/pets/plants.</em>

**WereSoCool** is a programming language for composing microtonal music geometrically. This language doesn't require familiarity with either microtonal music or computer programming, but experience with either will certainly help. 

## Installation:

### From Source:
You'll need Rust and optionally Just.

Rust: Rust is a great language. Install it with [Rustup](https://www.rust-lang.org/en-US/install.html).

You can install from this source code by cloning this repeo and then running:

1) If you have `just` installed, run `just install`
    - Just is a command runner. Learn how to install just [here](https://github.com/casey/just).

2) Or run `cargo install --path .` from the root of the repo.

### Macos:

`brew install weresocool`

### Arch Linux:

Available on the AUR [here](https://aur.archlinux.org/packages/weresocool).

### Cargo:

WereSoCool can be installed on any unix system via cargo. You'll also need to install the system dependancies listed in the development section. 

`cargo install weresocool`

### Windows

This software runs on Windows, but in a slightly limited capacity. On Windows, this software is compiled without mp3 and oggvorbis support. I mostly work on unix and linux machines, so I feel a bit out of my depths in Windows land. If you'd like to help work on the Windows implementation, please reach out. 

You can install WereSoCool from source or via cargo. See above. 

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
`brew install lame libvorbis`

##### Arch Linux:
`sudo pacman -S lame vorbis-tools`

##### Ubuntu:
`sudo apt-get lame libmp3lame-dev rpm libasound2-dev vorbis-tools`

### Build
`just build`

## Run Tests:
`just test`

## Special Thanks:
This wouldn't exist in a million years if it wasn't for Antonis Stampoulis'
help with language design or the help of friends/programmers like
Sönke Hahn, Hao Lian, Catharine M, Matt Handler, Lee Pender, Amanda Doucette, Khaled Alquaddoomi, Alex Kestner, everyone else that has sat down to program with me.
Of course, special thanks to Maria for always listening to my new_weird_sounds
and programming problems. - Danny

![WereSoCool](../main/imgs/cover.png)

Copyright (C) 2022 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).


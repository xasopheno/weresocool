# **WereSoCool**
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for compositing mictoronal music built in Rust. You might find this language useful if you want to make cool sounds and impress your friends/pets/plants. 

This language does not require familiarity with either microtonal music or computer programming, but experience with either will certainly help. 

The **WereSoCool CLI** is availble on **macOS**, **Linux** and **Windows**.  You can also explore the language in a Firefox or Chrome browser on a desktop computer in the [WereSoCool Playground](https://www.weresocool.org/playground). Live coding in the browser currently only works on desktop in a Firefox or Chrome browser. On mobile, you can still view the tutorials, but you won't be able to hear anything.

## Documentation:
If you want to learn how to make cool sounds using WereSoCool, you'll find cool documentation at [weresocool.org](https://www.weresocool.org/).

My recommended approach to learning the language is to do the tutorials in order and write your own composition after completing each one. I'm currently working on additional documentation as well as a record I've made using WereSoCool featuring a great band. Stay tuned. 

## Installation:

### Macos:

`brew tap xasopheno/weresocool && brew install weresocool`

### Arch Linux:

Available on the AUR [here](https://aur.archlinux.org/packages/weresocool).

### Cargo:

WereSoCool can be installed on macos, linux, and windows via cargo. You'll also need to install the system dependancies listed in the development section. 

 Install cargo by installing [Rustup](https://www.rust-lang.org/en-US/install.html).

`cargo install weresocool`

### Windows

This software runs on Windows, but in a slightly limited capacity. On Windows, this software is compiled without mp3 and oggvorbis support. I mostly work on macos and linux machines, so I feel a bit out of my depths in Windows land. If you'd like to help work on the Windows implementation, please reach out. 

You can install WereSoCool from source or via cargo. 

### From Source:
You'll need Cargo and Just.

Rust: Cargo is the rust package manager for Rust. Install cargo by installing [Rustup](https://www.rust-lang.org/en-US/install.html).

You can install from this source code by cloning this repo and then running:

You'll also need to install [Just](https://github.com/casey/just) - a cool cross-platform command runner. 

You can install from this source code by cloning this repo and then running:

`just install`


### WereSoCool CLI

```
Usage: weresocool [COMMAND]

Commands:
  new    Create a new .socool file from the template
  play   Render a .socool file.
    --watch                    On file save, the composition will be re-rendered
  demo   Hear a cool sound
  print  Print a .socool composition to a file
    --output_dir <output_dir>
    --wav                      print a wav file (default)
    --mp3                      print an mp3
    --oggvorbis                print an oggvorbis file
    --csv                      print a csv file
    --json                     print a json file
    --stems                    print stems as a zip file
    --sound                    print all sound file types
    --all                      print all file types
  help   Help of the given subcommand(s)
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
This wouldn't exist in a million years if it wasn't for Antonis Stampoulis' help with language design or the help of friends/programmers like Sönke Hahn, Hao Lian, Catharine M, Matt Handler, Alex David, Lee Pender, Amanda Doucette, Khaled Alquaddoomi, Alex Kestner, everyone else that has sat down to program with me.

Of course, special thanks to Maria for always listening to my new_weird_sounds
and programming problems. - Danny

Copyright (C) 2022 - Danny Meyer

This program is free software, licensed under the GPLv3 (see LICENSE).


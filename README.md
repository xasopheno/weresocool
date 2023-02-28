# **WereSoCool**
![Cool Tests](https://github.com/xasopheno/WereSoCool/workflows/Cool%20Tests/badge.svg)

A language for composing mictoronal music built in Rust. You might find this language useful if you want to make cool sounds and impress your friends/pets/plants. 

This language is designed to be easy to use, and you don't need any prior knowledge of microtonal music or computer programming. However, experience in either area will certainly be helpful.

The **WereSoCool CLI** is availble on **macOS**, **Linux** and **Windows**.  You can also explore the language in a Firefox or Chrome browser on a desktop computer in the [WereSoCool Playground](https://www.weresocool.org/playground). Live coding in the browser currently only works on desktop in a Firefox or Chrome browser. 

## Documentation:
If you want to learn how to make cool sounds using WereSoCool, you'll find cool documentation in English, Portuguêse, and Español
  at [weresocool.org](https://www.weresocool.org/).


My recommend following the tutorials in order and writing your own composition after completing each one. Additional documentation is currently being worked on, as well as a record featuring a great band, so stay tuned. 

On mobile, you can still view the tutorials, but you won't be able to hear anything.

## Installation:

### Macos:

`brew tap xasopheno/weresocool && brew install weresocool`

### Arch Linux:

Available on the AUR [here](https://aur.archlinux.org/packages/weresocool).

### Cargo:

WereSoCool can be installed on macos, linux, and windows via cargo. You'll also need to install the "Necessary Dependancies".

 Install cargo by installing [Rustup](https://www.rust-lang.org/en-US/install.html).

`cargo install weresocool`

#### Necessary Dependancies
Macos:
`brew install lame libvorbis`

Arch Linux (ALSA):
`sudo pacman -S lame vorbis-tools`

Ubuntu (ALSA):
`sudo apt-get lame vorbis-tools`

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
`sudo apt-get lame vorbis-tools`

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


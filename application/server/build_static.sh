#!/usr/bin/env bash

if [ "$(uname)" == "Darwin" ]; then
  echo "DARWIN...will be built statically."
 PKG_CONFIG_PATH='/usr/local/opt/' PORTAUDIO_ONLY_STATIC='true' RUSTFLAGS='-L/usr/local/opt/lame/lib/ -L/usr/local/opt/portaudio/lib/' cargo build --release
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
  echo "LINUX...will NOT build built statically."
  cargo build --release
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW32_NT" ]; then
  echo "Does not work on WINDOWS"
  exit 1
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
  echo "Does not work on WINDOWS"
  exit 1
fi



#!/usr/bin/env bash

if [ "$(uname)" == "Darwin" ]; then
  PORTAUDIO_ONLY_STATIC=true RUSTFLAGS='-L/usr/local/opt/portaudio/lib/ -L/usr/local/opt/lame/lib/' cargo build --release
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
  echo "Linux...will not build built statically."
  cargo build --release
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW32_NT" ]; then
  echo "Does not work on Windows"
  exit 1
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
  echo "Does not work on Windows"
  exit 1
fi



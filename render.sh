#!/usr/bin/env sh


FILENAME=$1
MP3=${FILENAME##*/}.mp3
JSON=${FILENAME##*/}.socool.json
#rm ../wereso_visible/public/songs/${MP3, JSON}
#rm renders/${MP3, JSON}

cargo run --release --bin wsc songs/$FILENAME.socool --print

cargo run --release --bin wsc songs/$FILENAME.socool --json

#rsync --inplace renders/ ../wereso_visible/public/songs
mv renders/${MP3} ../wereso_visible/public/songs/${MP3}
mv renders/${JSON} ../wereso_visible/public/songs/${JSON}




#!/usr/bin/env sh

while read SONG; do
    cargo run --release --bin wsc songs/$SONG.socool --csv
done < to_csv.txt

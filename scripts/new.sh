FILENAME=$1

cat songs/template.socool  > songs/$FILENAME.socool
nvim songs/$FILENAME.socool



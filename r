#!/usr/bin/env sh

FILENAME=$1
fd | entr -crs "echo | ./render $FILENAME"


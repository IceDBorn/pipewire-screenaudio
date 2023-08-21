#!/usr/bin/env bash

scriptRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"

cd $scriptRoot

# Watch for file changes
ls | entr -sc "npx vite build && notify-send 'Pipewire Screenaudio' 'Extension rebuilt'"


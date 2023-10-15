#!/usr/bin/env bash

scriptRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"

cd $scriptRoot

# Watch for file changes
npx nodemon -e js,jsx --ignore ./dist/ --exec "npx vite build && notify-send -i $scriptRoot/../assets/icons/icon.svg 'Pipewire Screenaudio' 'Extension rebuilt'"

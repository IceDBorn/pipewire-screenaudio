#!/usr/bin/env bash

micPid=$1
micId=$2
mainPid=$$

isCapturing=0

function countVirtmicLinks () {
  pactl subscribe | grep --line-buffered $micId | (
    while read -r line; do
      ((echo "$line" | grep 'remove') && (
        pstree -A -p $mainPid | grep -Eow '[0-9]+' | xargs kill
      )) > /dev/null

      pw-link -l | grep pipewire-screenaudio | wc -l
    done
  )
}

function mainLoop () {
  while read -r line; do
    if [ $line -eq 8 ] && [ $isCapturing -eq 0 ]; then
      isCapturing=1
    fi

    if [ $line -eq 4 ] && [ $isCapturing -eq 1 ]; then
      kill $micPid
    fi
  done
}

countVirtmicLinks | mainLoop

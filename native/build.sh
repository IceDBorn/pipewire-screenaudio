#!/usr/bin/env bash

set -e

cmake -B build -S .
cmake --build build

mkdir -p out
cp build/screenaudio-mic/screenaudio-mic out/

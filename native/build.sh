#!/usr/bin/env bash

set -e

cmake -B build -S .
cmake --build build

mkdir -p out
cp build/pipewire-screenaudio/pipewire-screenaudio out/

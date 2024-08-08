#!/usr/bin/env bash

set -e

projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"

( cd $projectRoot/native/connector-rs ; cargo build --release)

mkdir -p $HOME/.local/lib/pipewire-screenaudio
mkdir -p $HOME/.local/share/pipewire-screenaudio/
mkdir -p $HOME/.local/bin

cp -r $projectRoot/native/native-messaging-hosts $HOME/.local/share/pipewire-screenaudio/
cp $projectRoot/native/connector-rs/target/release/connector-rs $HOME/.local/lib/pipewire-screenaudio/

cp $projectRoot/native/screenaudioctl.sh $HOME/.local/bin/screenaudioctl

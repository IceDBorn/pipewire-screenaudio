#!/usr/bin/env bash

export LC_ALL=C
export PROJECT_ROOT="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; cd .. ; pwd -P )"
source $PROJECT_ROOT/connector/util.sh

cmd=$1
args=$2

json="{ \"cmd\": \"$cmd\", \"args\": [$args] }"

echo $cmd $args $json >/dev/stderr

UtilTextToMessage "$json" \
  | (
    cd $PROJECT_ROOT/connector-rs
    if [[ "$DEBUG" -eq 1 ]]; then
      export RUST_BACKTRACE=1
      export RUST_LOG=debug
    fi
    cargo run
  ) \
  | ( head -c 4 >/dev/null ; jq )

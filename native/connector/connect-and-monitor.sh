#!/usr/bin/env bash

# This script finds and connects targetNodeSerial to virtual mic

virtmicPortFlId=$1
virtmicPortFrId=$2
targetNodeSerial=$3

source $PROJECT_ROOT/connector/util.sh

exec 2>>`UtilGetLogPathForFile $(basename $0)`
set -e

# Cleanup on exit
trap "trap - SIGTERM && kill -- -$$" SIGTERM EXIT

{
  # Watch for new nodes to connect
  [[ ! "$targetNodeSerial" -eq "-1" ]] && {
      UtilLog "[connect-and-monitor.sh] [Entering Single Node Mode] Serial: $targetNodeSerial"
      ./find-ports-by-node-serial.lua targetSerial="$targetNodeSerial"
    } || {
      UtilLog "[connect-and-monitor.sh] [Entering All Desktop Audio Mode] Serial: $targetNodeSerial"
      ./monitor-new-nodes.lua
    }
} | {
    while read -r flId frId; do (
        UtilLog "[connect-and-monitor.sh] [Got Ports IDs] FL IDs: $flId, FR IDs: $frId"
        linkLog="[connect-and-monitor.sh] [Linking Ports]"

        # Connect new node to virtmic
        # Link FL
        linkLog="$linkLog ($flId -> $virtmicPortFlId"
        pw-link $flId $virtmicPortFlId && {
            linkLog="$linkLog Success)"
        } || {
            linkLog="$linkLog Fail)"
        }

        # Link FR
        linkLog="$linkLog ($frId -> $virtmicPortFrId"
        pw-link $frId $virtmicPortFrId && {
            linkLog="$linkLog Success)"
        } || {
            linkLog="$linkLog Fail)"
        }

        UtilLog "$linkLog"
    ) done
} &

wait

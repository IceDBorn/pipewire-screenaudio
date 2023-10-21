#!/usr/bin/env bash

export LC_ALL=C
export PROJECT_ROOT="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; cd .. ; pwd -P )"
source $PROJECT_ROOT/connector/util.sh

exec 2>>`UtilGetLogPathForFile $(basename $0)`

function GetVersion () {
  UtilTextToMessage "{\"version\":\"$VERSION\"}"
}

function GetSessionType () {
  type=`[[ -z "$WAYLAND_DISPLAY" ]] && echo "x11" || echo "wayland"`
  UtilTextToMessage "{\"type\": \"$type\"}"
}

function GetNodes () {
  local nodes=`./list-nodes.lua`
  UtilTextToMessage "$nodes"
  exit
}

function StartPipewireScreenAudio () {
  setsid $PROJECT_ROOT/connector/virtmic.sh &

  sleep 1
  local micId=`./find-screenaudio-node.lua`

  UtilTextToMessage '{"micId":'$micId'}'
  exit
}

function SetSharingNode () {
  local node=`UtilGetArg 'node'`
  local micId=`UtilGetArg 'micId'`
  local fifoPath=`UtilGetFifoPath "$micId"`

  if [ -e "$fifoPath" ]; then
    echo "$node" >> "$fifoPath"
  fi

  UtilTextToMessage '{"success":true}'
  exit
}

function StopPipewireScreenAudio () {
  local micId=`UtilGetArg 'micId'`

  if [ ! "`pw-cli info "$micId" 2>/dev/null | wc -l`" -eq "0" ]; then
    [ "`pw-cli destroy "$micId" 2>&1 | wc -l`" -eq "0" ] &&
      UtilTextToMessage '{"success":true}' && exit
  fi

  UtilTextToMessage '{"success":false}'
  exit
}

function IsPipewireScreenAudioRunning () {
  local micId=`UtilGetArg 'micId'`

  if pw-cli info "$micId" 2>/dev/null | grep 'node.name' | grep "$VIRTMIC_NODE_NAME" >/dev/null; then
    UtilTextToMessage '{"isRunning":true}' && exit
  fi

  UtilTextToMessage '{"isRunning":false}'
  exit
}

UtilGetPayload

case "$cmd" in
  'GetVersion')
    GetVersion
    ;;
  'GetSessionType')
    GetSessionType
    ;;
  'GetNodes')
    GetNodes
    ;;
  'StartPipewireScreenAudio')
    StartPipewireScreenAudio
    ;;
  'SetSharingNode')
    SetSharingNode
    ;;
  'IsPipewireScreenAudioRunning')
    IsPipewireScreenAudioRunning
    ;;
  'StopPipewireScreenAudio')
    StopPipewireScreenAudio
    ;;
esac

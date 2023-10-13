#!/usr/bin/env bash

export LC_ALL=C
projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; cd .. ; pwd -P )"
source $projectRoot/connector/util.sh

function GetVersion () {
  UtilTextToMessage '{"version":"0.3.2"}'
}

function GetSessionType () {
  # https://unix.stackexchange.com/a/325972
  type=`loginctl show-session $(awk '/tty/ {print $1}' <(loginctl)) -p Type | awk -F= '{print $2}'`
  UtilTextToMessage "{\"type\": \"$type\"}"
}

function GetNodes () {
  local nodes=`pw-dump | jq -c '
    [{
      "properties": {
        "media.name": "[All Desktop Audio]",
        "application.name": "",
        "object.serial": -1
      }
    }] + [ .[] |
      select(.info.props["media.class"] == "Stream/Output/Audio") |
      .["properties"] = .info.props |
      del(.info)
    ]
  '`
  UtilTextToMessage "$nodes"
  exit
}

function StartPipewireScreenAudio () {
  setsid $projectRoot/connector/virtmic.sh >/dev/null 2>&1 &

  sleep 1
  local micId=`
    pw-dump |
      jq -c '[ .[] | select(.info.props["node.name"] == "pipewire-screenaudio") ][0].id'
  `

  UtilTextToMessage '{"micId":'$micId'}'
  exit
}

function SetSharingNode () {
  local args="$1"

  local node=`UtilGetArg 'node'`
  local micId=`UtilGetArg 'micId'`
  local fifoPath=`UtilGetFifoPath "$micId"`

  if [ -e "$fifoPath" ]; then
    echo "$node" >> "$fifoPath"
  fi

  toMessage '{"success":true}'
  exit
}

function StopPipewireScreenAudio () {
  local args="$1"
  local micId=`UtilGetArg 'micId'`

  if [ ! "`pw-cli info "$micId" 2>/dev/null | wc -l`" -eq "0" ]; then
    [ "`pw-cli destroy "$micId" 2>&1 | wc -l`" -eq "0" ] &&
      UtilTextToMessage '{"success":true}' && exit
  fi

  UtilTextToMessage '{"success":false}'
  exit
}

function IsPipewireScreenAudioRunning () {
  local args="$1"
  local micId=`UtilGetArg 'micId'`

  if pw-cli info "$micId" 2>/dev/null | grep 'node.name' | grep 'pipewire-screenaudio' >/dev/null; then
    toMessage '{"isRunning":true}' && exit
  fi

  UtilTextToMessage '{"isRunning":false}'
  exit
}

UtilGetPayload

case $cmd in
  'GetVersion')
    GetVersion
    ;;
  'GetSessionType')
    GetSessionType
    ;;
  'GetNodes')
    GetNodes "$args"
    ;;
  'StartPipewireScreenAudio')
    StartPipewireScreenAudio
    ;;
  'SetSharingNode')
    SetSharingNode "$args"
    ;;
  'IsPipewireScreenAudioRunning')
    IsPipewireScreenAudioRunning "$args"
    ;;
  'StopPipewireScreenAudio')
    StopPipewireScreenAudio "$args"
    ;;
esac

#!/usr/bin/env bash

export LC_ALL=C
export PROJECT_ROOT="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; cd .. ; pwd -P )"
source $PROJECT_ROOT/connector/util.sh

exec 2>>`UtilGetLogPathForFile $(basename $0)`

function GetVersion () {
  echo "{\"version\":\"$VERSION\"}"
  exit
}

function GetSessionType () {
  type=`[[ -z "$WAYLAND_DISPLAY" ]] && echo "x11" || echo "wayland"`
  echo "{\"type\": \"$type\"}"
  exit
}

function GetNodes () {
  local nodes=`pw-dump | jq -c '
    [ .[] |
      select(.info.props["media.class"] == "Stream/Output/Audio") |
      .["properties"] = .info.props |
      del(.info)
    ]
  '`
  echo "$nodes"
  exit
}

function StartPipewireScreenAudio () {
  setsid $PROJECT_ROOT/connector/virtmic.sh &

  sleep 1
  local micId=`
    pw-dump |
      jq -c "[ .[] | select(.info.props[\"node.name\"] == \"$VIRTMIC_NODE_NAME\") ][0].id"
  `

  echo '{"micId":'$micId'}'
  exit
}

function SetSharingNode () {
  local fifoPath=`UtilGetFifoPath "$args_micId"`

  if [ -e "$fifoPath" ]; then
    echo "$args_nodes" >> "$fifoPath"
  fi

  echo '{"success":true}'
  exit
}

# TODO Implement as standalone function
function ShareAllDesktopAudio () {
  args_nodes='[-1]'
  SetSharingNode
}

function StopPipewireScreenAudio () {
  if [ ! "`pw-cli info "$args_micId" 2>/dev/null | wc -l`" -eq "0" ]; then
    [ "`pw-cli destroy "$args_micId" 2>&1 | wc -l`" -eq "0" ] &&
      echo '{"success":true}' && exit
  fi

  echo '{"success":false}'
  exit
}

function IsPipewireScreenAudioRunning () {
  if pw-cli info "$args_micId" 2>/dev/null | grep 'node.name' | grep "$VIRTMIC_NODE_NAME" >/dev/null; then
    echo '{"isRunning":true}' && exit
  fi

  echo '{"isRunning":false}'
  exit
}

UtilReadPayload "$1"

case "$cmd" in
  'GetVersion')
    UtilCallCommand GetVersion
    ;;
  'GetSessionType')
    UtilCallCommand GetSessionType
    ;;
  'GetNodes')
    UtilCallCommand GetNodes
    ;;
  'StartPipewireScreenAudio')
    UtilCallCommand StartPipewireScreenAudio
    ;;
  'SetSharingNode')
    UtilCallCommand --required-args 'micId,nodes' SetSharingNode
    ;;
  'ShareAllDesktopAudio')
    UtilCallCommand --required-args 'micId' ShareAllDesktopAudio
    ;;
  'IsPipewireScreenAudioRunning')
    UtilCallCommand --required-args 'micId' IsPipewireScreenAudioRunning
    ;;
  'StopPipewireScreenAudio')
    UtilCallCommand --required-args 'micId' StopPipewireScreenAudio
    ;;
esac

#!/usr/bin/env bash

export LC_ALL=C
export PROJECT_ROOT="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; cd .. ; pwd -P )"
source $PROJECT_ROOT/connector/util.sh

exec 2>>`UtilGetLogPathForFile $(basename $0)`

function GetVersion () {
  echo "{\"version\":\"$VERSION\"}"
}

function GetSessionType () {
  type=`[[ -z "$WAYLAND_DISPLAY" ]] && echo "x11" || echo "wayland"`
  echo "{\"type\": \"$type\"}"
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
  local nodes=`UtilGetArg 'nodes'`
  local micId=`UtilGetArg 'micId'`
  local fifoPath=`UtilGetFifoPath "$micId"`

  if [ -e "$fifoPath" ]; then
    echo "$nodes" >> "$fifoPath"
  fi

  echo '{"success":true}'
  exit
}

# TODO Implement as standalone function
function ShareAllDesktopAudio () {
  local micId=`UtilGetArg 'micId'`
  args="{\"micId\":\"$micId\",\"nodes\":[-1]}"
  SetSharingNode
}

function StopPipewireScreenAudio () {
  local micId=`UtilGetArg 'micId'`

  if [ ! "`pw-cli info "$micId" 2>/dev/null | wc -l`" -eq "0" ]; then
    [ "`pw-cli destroy "$micId" 2>&1 | wc -l`" -eq "0" ] &&
      echo '{"success":true}' && exit
  fi

  echo '{"success":false}'
  exit
}

function IsPipewireScreenAudioRunning () {
  local micId=`UtilGetArg 'micId'`

  if pw-cli info "$micId" 2>/dev/null | grep 'node.name' | grep "$VIRTMIC_NODE_NAME" >/dev/null; then
    echo '{"isRunning":true}' && exit
  fi

  echo '{"isRunning":false}'
  exit
}

UtilReadPayload "$1"

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
  'ShareAllDesktopAudio')
    ShareAllDesktopAudio
    ;;
  'IsPipewireScreenAudioRunning')
    IsPipewireScreenAudioRunning
    ;;
  'StopPipewireScreenAudio')
    StopPipewireScreenAudio
    ;;
esac

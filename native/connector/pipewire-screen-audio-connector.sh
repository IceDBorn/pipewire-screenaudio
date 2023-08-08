#!/usr/bin/env bash

export LC_ALL=C
projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; cd .. ; pwd -P )"

function intToBin () {
  printf '%08x' $1 |                # Convert integer to 8 digit hex
    sed 's/\(..\)/\1 /g' |          # Split hex to pairs (bytes)
    awk '{printf $4 $3 $2 $1}' |    # Reverse order of bytes
    sed 's/\(..\)\s*/\\\\x\1/g' |   # Prefix bytes with \\x
    xargs -I {} sh -c "printf '{}'" # Return raw bytes
}

function binToInt () {
  head -c 4 |                           # Read 4 bytes
    hexdump |                           # Read raw bytes as hex
    head -n 1 |                         # Discard new line
    awk '{print "0x"$3$2}' |            # Format hex number
    xargs -I {} sh -c 'echo $(( {} ))'  # Return int
}

function toMessage () {
  local message="$1"
  local messageLength=`echo -n "$message" | wc -c`

  intToBin $messageLength
  echo -n "$message"
}

function GetVersion () {
  toMessage '{"version":"0.2.0"}'
}

function GetSessionType () {
  # https://unix.stackexchange.com/a/325972
  type=`loginctl show-session $(awk '/tty/ {print $1}' <(loginctl)) -p Type | awk -F= '{print $2}'`
  toMessage "{\"type\": \"$type\"}"
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
  toMessage "$nodes"
  exit
}

function StartPipewireScreenAudio () {
  setsid $projectRoot/connector/virtmic.sh >/dev/null 2>&1 &

  sleep 1
  local micId=`
    pw-dump |
      jq -c '[ .[] | select(.info.props["node.name"] == "pipewire-screenaudio") ][0].id'
  `

  toMessage '{"micId":'$micId'}'
  exit
}

function SetSharingNode () {
  local args="$1"

  local node=`echo $args | jq -r '.[].node' | head -n 1`
  local virtmicId=`echo $args | jq -r '.[].micId' | head -n 1`
  fifoPath="$XDG_RUNTIME_DIR/pipewire-screenaudio-set-node-$virtmicId"

  if [ -e "$fifoPath" ]; then
    echo "$node" >> "$fifoPath"
  fi

  exit
}

function StopPipewireScreenAudio () {
  local args="$1"
  local micId=`echo $args | jq '.[].micId' | xargs | head -n 1`

  if [ ! "`pw-cli info "$micId" 2>/dev/null | wc -l`" -eq "0" ]; then
    [ "`pw-cli destroy "$micId" 2>&1 | wc -l`" -eq "0" ] &&
      toMessage '{"success":true}' && exit
  fi

  toMessage '{"success":false}'
  exit
}

function IsPipewireScreenAudioRunning () {
  local args="$1"
  local micId=`echo $args | jq '.[].micId' | xargs | head -n 1`

  if [ ! "`pw-cli info "$micId" 2>/dev/null | wc -l`" -eq "0" ]; then
    toMessage '{"isRunning":true}' && exit
  fi

  toMessage '{"isRunning":false}'
  exit
}

payloadLength=`binToInt`
payload=`head -c "$payloadLength"`

cmd=`echo "$payload" | jq -r .cmd`
args=`echo "$payload" | jq .args`

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

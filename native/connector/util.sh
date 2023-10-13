VERSION="0.3.2"
VIRTMIC_NODE_NAME='pipewire-screenaudio'
FIFO_PATH_PREFIX="$XDG_RUNTIME_DIR/pipewire-screenaudio-set-node-"
LOG_PATH="$XDG_RUNTIME_DIR/pipewire-screenaudio.log"

CURRENT_PID=$$

set -e

UtilGetFifoPath () {
  local virtmicId="$1"
  printf "$FIFO_PATH_PREFIX$virtmicId"
}

function UtilBinToInt () {
  head -c 4 |                             # Read 4 bytes
    hexdump |                             # Read raw bytes as hex
    head -n 1 |                           # Discard new line
    awk '{print "0x"$3$2}' |              # Format hex number
    xargs -I {} bash -c 'echo $(( {} ))'  # Return int
}

function UtilIntToBin () {
  printf '%08x' $1 |                  # Convert integer to 8 digit hex
    sed 's/\(..\)/\1 /g' |            # Split hex to pairs (bytes)
    awk '{printf $4 $3 $2 $1}' |      # Reverse order of bytes
    sed 's/\(..\)\s*/\\\\x\1/g' |     # Prefix bytes with \\x
    xargs -I {} bash -c "printf '{}'" # Return raw bytes
}

function UtilTextToMessage () {
  local message="$1"
  local messageLength=`echo -n "$message" | wc -c`

  UtilLog "[util.sh] [Sending Message] $message Length: $messageLength"

  UtilIntToBin $messageLength
  echo -n "$message"
}

function UtilGetPayload () {
  payloadLength=`UtilBinToInt`
  UtilLog "[util.sh] [Reading Bytes] $payloadLength"

  payload=`head -c "$payloadLength"`
  UtilLog "[util.sh] [Got Payload] $payload"

  cmd=`echo "$payload" | jq -r .cmd`
  UtilLog "[util.sh] [Got Cmd] $cmd"

  args=`echo "$payload" | jq .args`
  UtilLog "[util.sh] [Got Args] $args"
}

function UtilGetArg () {
  local field="$1"
  UtilLog "[util.sh] [Reading Arg] $field"
  local arg=`echo $args | jq -r ".[].$field" | head -n 1`
  UtilLog "[util.sh] [Arg Value] `[ "$arg" = "" ] && printf 'null' || printf "$arg"`"
  printf $arg
}

function UtilLog () {
  echo "$@" >> $LOG_PATH
  # notify-send "$@"
}

function UtilGetLogPathForFile () {
  mkdir -p "$LOG_PATH.d"
  echo "$LOG_PATH.d/$1.log"
}

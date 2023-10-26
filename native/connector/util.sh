VERSION="0.3.4"
VIRTMIC_NODE_NAME='pipewire-screenaudio'

TEMP_PATH_ROOT="$XDG_RUNTIME_DIR/pipewire-screenaudio"
FIFO_PATH="$TEMP_PATH_ROOT/fifos"
LOG_PATH="$TEMP_PATH_ROOT/logs"

mkdir -p $LOG_PATH
MAIN_LOG_PATH="$LOG_PATH/main.log"

CURRENT_PID=$$

set -e

UtilGetFifoPath () {
  local virtmicId="$1"
  mkdir -p $FIFO_PATH
  printf "$FIFO_PATH/$virtmicId"
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
  local arg=`echo "$args" | jq -rc ".[].$field" | head -n 1`
  UtilLog "[util.sh] [Arg Value] `[ "$arg" = "" ] && printf 'null' || printf "%s" "$arg"`"
  printf "%s" "$arg"
}

function UtilLog () {
  echo "$@" >> $MAIN_LOG_PATH
  # notify-send "$@"
}

function UtilGetLogPathForFile () {
  mkdir -p "$LOG_PATH/file"
  echo "$LOG_PATH/file/$1.log"
}

function UtilGetTempFile () {
  mkdir -p "$XDG_RUNTIME_DIR/tmp"
  mktemp -p "$XDG_RUNTIME_DIR/tmp"
}

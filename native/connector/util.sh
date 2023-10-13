VERSION="0.3.2"
FIFO_PATH_PREFIX="$XDG_RUNTIME_DIR/pipewire-screenaudio-set-node-"

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

  UtilIntToBin $messageLength
  echo -n "$message"
}

function UtilGetPayload () {
  payloadLength=`UtilBinToInt`
  payload=`head -c "$payloadLength"`

  cmd=`echo "$payload" | jq -r .cmd`
  args=`echo "$payload" | jq .args`
}

function UtilGetArg () {
  local field="$1"
  echo $args | jq -r ".[].$field" | head -n 1
}

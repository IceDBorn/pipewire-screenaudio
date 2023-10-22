#!/usr/bin/env bash

source $PROJECT_ROOT/connector/util.sh

exec 1>>`UtilGetLogPathForFile $(basename $0)` 2>&1


# Check if $VIRTMIC_NODE_NAME exists, and create it
virtmicId=`./find-screenaudio-node.lua`
if [[ -z "$virtmicId" ]]; then
    pw-cli create-node adapter "{ factory.name=support.null-audio-sink node.name=$VIRTMIC_NODE_NAME media.class=Audio/Source/Virtual object.linger=1 audio.position=[FL,FR] }"
    virtmicId=`./wait-screenaudio-node.lua`
    UtilLog "[virtmic.sh] [Created Node] $VIRTMIC_NODE_NAME (id = $virtmicId)"
else
    UtilLog "[virtmic.sh] [Found Node] $VIRTMIC_NODE_NAME (id = $virtmicId)"
fi

read -r virtmicPortFlId virtmicPortFrId <<< `./find-ports-by-node-id.lua targetId="$virtmicId"`

UtilLog "[virtmic.sh] [Got Ports] FL: $virtmicPortFlId"
UtilLog "[virtmic.sh] [Got Ports] FR: $virtmicPortFrId"

fifoPath=`UtilGetFifoPath "$virtmicId"`
mkfifo "$fifoPath"
UtilLog "[virtmic.sh] [Created FIFO] $fifoPath"

# Cleanup on exit
trap "rm $fifoPath; kill -- -$CURRENT_PID" EXIT

function disconnectInputs() {
    node=$1
    UtilLog "[virtmic.sh] [Disconnecting Inputs] Node: $node"
    ./list-node-inputs.lua nodeId="$node" | xargs -r -n1 pw-cli destroy
    UtilLog "[virtmic.sh] [Disconnected Inputs] Node: $node"
}

# Listen to selected node change
tail -f "$fifoPath" | {
    # Kill connect-and-monitor.sh process if it's still alive
    function killMonitor() {
        if [ ! -z "$monitorProcess" ]; then
            UtilLog "[virtmic.sh] [Killing] PID: $monitorProcess"
            kill $monitorProcess || :
        fi
    }

    # Kill monitor process when this background process gets killed
    trap "killMonitor" EXIT

    # Read the new target node
    while read -r targetNodeSerial; do
        UtilLog "[virtmic.sh] [Got FIFO Data] $targetNodeSerial"
        killMonitor
        disconnectInputs "$virtmicId"
        setsid bash -- connect-and-monitor.sh "$virtmicPortFlId" "$virtmicPortFrId" "$targetNodeSerial" &
        monitorProcess=$!
        UtilLog "[virtmic.sh] [Started Background Task] Script: connect-and-monitor.sh, PID: $monitorProcess"
    done
} &

# Send SIGTERM to this process when the virtmic gets removed
{
  ./monitor-node-deletion.lua targetId="$virtmicId"
  UtilLog "[virtmic.sh] [Killing] PID: $CURRENT_PID (self)"
  kill -SIGTERM $CURRENT_PID
} &

wait

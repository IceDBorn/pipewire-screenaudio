#!/usr/bin/env bash

source $PROJECT_ROOT/connector/util.sh

exec 1>>`UtilGetLogPathForFile $(basename $0)` 2>&1

# Get all nodes to check if $VIRTMIC_NODE_NAME exists, and create it
pw-dump |
    jq --exit-status -c -s "flatten(1) | [ .[] | select(.info.props[\"node.name\"] == \"$VIRTMIC_NODE_NAME\") ][0]" >/dev/null && (
      UtilLog "[virtmic.sh] [Found Node] $VIRTMIC_NODE_NAME"
    ) || (
      pw-cli create-node adapter "{ factory.name=support.null-audio-sink node.name=$VIRTMIC_NODE_NAME media.class=Audio/Source/Virtual object.linger=1 audio.position=[FL,FR] }"
      UtilLog "[virtmic.sh] [Created Node] $VIRTMIC_NODE_NAME"
    )

fullDumpFile=`UtilGetTempFile`

# === Collect required data from PipeWire === #
# Get all nodes again for further processing
pw-dump | jq -s "flatten(1)" > $fullDumpFile
UtilLog "[virtmic.sh] [Got Dump] File: $fullDumpFile"

# Get id and ports of $VIRTMIC_NODE_NAME
virtmicId=`cat "$fullDumpFile" | jq -c "[ .[] | select(.info.props[\"node.name\"] == \"$VIRTMIC_NODE_NAME\") ][0].id"`
UtilLog "[virtmic.sh] [Got Id] $virtmicId"

virtmicPortsFile=`UtilGetTempFile`
cat "$fullDumpFile" | jq -c "[ .[] | select(.info.direction == \"input\") | select(.info.props[\"node.id\"] == $virtmicId) ]" > $virtmicPortsFile
UtilLog "[virtmic.sh] [Got Ports] File: $virtmicPortsFile"

virtmicPortFlId=`cat "$virtmicPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][0].id"`
virtmicPortFrId=`cat "$virtmicPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][0].id"`
UtilLog "[virtmic.sh] [Got Ports] FL: $virtmicPortFlId"
UtilLog "[virtmic.sh] [Got Ports] FR: $virtmicPortFrId"

rm $virtmicPortsFile
UtilLog "[virtmic.sh] [Cleared Ports] File: $virtmicPortsFile"

fifoPath=`UtilGetFifoPath "$virtmicId"`
mkfifo "$fifoPath"
UtilLog "[virtmic.sh] [Created FIFO] $fifoPath"

# Cleanup on exit
trap "rm $fullDumpFile $fifoPath; kill -- -$CURRENT_PID" EXIT

function disconnectInputs() {
    node=$1
    UtilLog "[virtmic.sh] [Disconnecting Inputs] Node: $node"
    pw-dump | jq -s -r "
        flatten(1) |
        [
            .[] |
            select(.info[\"input-node-id\"] == $node) |
            .id
        ] |
        .[]
    " | xargs -r -n1 pw-cli destroy
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

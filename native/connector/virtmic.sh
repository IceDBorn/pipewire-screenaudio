#!/usr/bin/env bash

virtmicNode='pipewire-screenaudio'

myPid=$$

set -e

# Get all nodes to check if $virtmicNode exists, and create it
pw-dump |
    jq --exit-status -c -s "flatten(1) | [ .[] | select(.info.props[\"node.name\"] == \"$virtmicNode\") ][0]" >/dev/null ||
    pw-cli create-node adapter "{ factory.name=support.null-audio-sink node.name=$virtmicNode media.class=Audio/Source/Virtual object.linger=1 audio.position=[FL,FR] }"

fullDumpFile=`mktemp`

# === Collect required data from PipeWire === #
# Get all nodes again for further processing
pw-dump | jq -s "flatten(1)" > $fullDumpFile

# Get id and ports of $virtmicNode
virtmicId=`cat "$fullDumpFile" | jq -c "[ .[] | select(.info.props[\"node.name\"] == \"$virtmicNode\") ][0].id"`
virtmicPortsFile=`mktemp`
cat "$fullDumpFile" | jq -c "[ .[] | select(.info.direction == \"input\") | select(.info.props[\"node.id\"] == $virtmicId) ]" > $virtmicPortsFile
virtmicPortFlId=`cat "$virtmicPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][0].id"`
virtmicPortFrId=`cat "$virtmicPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][0].id"`
rm $virtmicPortsFile

fifoPath="$XDG_RUNTIME_DIR/pipewire-screenaudio-set-node-$virtmicId"
mkfifo "$fifoPath"

# Cleanup on exit
trap "rm $fullDumpFile $fifoPath; kill -- -$myPid" EXIT

function monitor-nodes() {
    tail -f /dev/null | pw-cli -m | grep --line-buffered -v 'pipewire.sec.label = "hex:'
}

function disconnectInputs() {
    node=$1
    pw-dump | jq -s -r "
        flatten(1) |
        [
            .[] |
            select(.info[\"input-node-id\"] == $node) |
            .id
        ] |
        .[]
    " | xargs -r -n1 pw-cli destroy
}

# Listen to selected node change
tail -f "$fifoPath" | {
    # Kill connect-and-monitor.sh process if it's still alive
    function killMonitor() {
        if [ ! -z "$monitorProcess" ]; then
            kill $monitorProcess || :
        fi
    }

    # Kill monitor process when this background process gets killed
    trap "killMonitor" EXIT

    # Read the new target node
    while read -r targetNodeSerial; do
        killMonitor
        disconnectInputs "$virtmicId"
        setsid bash -- connect-and-monitor.sh "$virtmicPortFlId" "$virtmicPortFrId" "$targetNodeSerial" &
        monitorProcess=$!
    done
} &

# Send SIGTERM to this process when the virtmic gets removed
monitor-nodes |
    stdbuf -o0 awk '/remote 0 removed global/ && /Node/' |
    grep --line-buffered -oP "id \\K$virtmicId" | {
    while read -r id; do
        kill -SIGTERM $myPid
    done
} &

wait

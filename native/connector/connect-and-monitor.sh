#!/usr/bin/env bash

# This script finds and connects targetNodeSerial to virtual mic

virtmicPortFlId=$1
virtmicPortFrId=$2
targetNodeSerial=$3


EXCLUDED_TARGETS='"AudioCallbackDriver"'

fullDumpFile=`mktemp`
streamsFile=`mktemp`
portsFile=`mktemp`


function monitor-nodes() {
    tail -f /dev/null | pw-cli -m | grep --line-buffered -v 'pipewire.sec.label = "hex:'
}

set -e

# Cleanup on exit
trap "rm $fullDumpFile $streamsFile $portsFile; trap - SIGTERM && kill -- -$$" SIGTERM EXIT

pw-dump | jq -s "flatten(1)" > $fullDumpFile
# Get streams from $fullDumpFile
cat "$fullDumpFile" | jq -c '[ .[] | select(.info.props["media.class"] == "Stream/Output/Audio") ]' > $streamsFile

# Get output ports of streams from $fullDumpFile
streamIds=`cat "$streamsFile" | jq -c '.[].id' | paste -sd ','`
cat "$fullDumpFile" | jq -c "[ .[] | select(.type == \"PipeWire:Interface:Port\") | select(.info.direction == \"output\") | select(.info.props[\"node.id\"] | contains($streamIds)) ]" > $portsFile
if [[ ! "$targetNodeSerial" -eq "-1" ]]; then
    # Get target node id from $streamsFile
    targetNodeId=`cat "$streamsFile" | jq -c "[ .[] | select(.info.props[\"object.serial\"] == $targetNodeSerial) ][0].id"`

    # Get target node ports ids from $portsFile
    targetPortsFile=`mktemp`
    cat "$portsFile" | jq -c "[ .[] | select(.info.props[\"node.id\"] == $targetNodeId) ]" > $targetPortsFile
    targetPortFlId=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][0].id"`
    targetPortFrId=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][0].id"`
    rm $targetPortsFile

    # Connect target to virtmic
    pw-link $targetPortFlId $virtmicPortFlId
    pw-link $targetPortFrId $virtmicPortFrId
else
    # Get target nodes ids to connect from $streamsFile
    targetNodesIds=`cat $streamsFile | jq -c "[ .[] | select(.info.props[\"media.name\"] | contains($EXCLUDED_TARGETS) | not) ][].id" | paste -sd ','`

    if [[ ! "$targetNodesIds" -eq "" ]]; then
        # Get target nodes ports ids from $portsFile
        targetPortsFile=`mktemp`
        cat "$portsFile" | jq -c "[ .[] | select(.info.props[\"node.id\"] | contains($targetNodesIds)) ]" > $targetPortsFile
        targetPortsFlIds=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][].id"`
        targetPortsFrIds=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][].id"`
        rm $targetPortsFile

        # Connect targets to virtmic
        echo "$targetPortsFlIds" | while read -r id; do pw-link $id $virtmicPortFlId; done
        echo "$targetPortsFrIds" | while read -r id; do pw-link $id $virtmicPortFrId; done
    fi

    # Watch for new nodes to connect
    monitor-nodes | {
        stdbuf -o0 awk '/remote 0 added global/ && /Node/' |
            grep --line-buffered -oP 'id \K\d+' |
            while read -r id; do (
                # Skip excluded targets
                pw-dump "$id" | jq --exit-status -c "
                    [
                        .[] |
                        select(.id == $id) |
                        select(.info.props[\"media.name\"] | contains($EXCLUDED_TARGETS) | not)
                    ][0].id
                " >/dev/null || exit 0

                # 1. Find the ports with node.id == $id
                # 2. Get only the FR and FL ports
                # 3. Sort by audio.channel (FR > FL)
                # 4. Return only the ids
                ids=`pw-dump | jq -s -c "
                    flatten(1) |
                    [
                        .[] |
                        select(.info.props[\"node.id\"] == $id) |
                        select(.info.props[\"audio.channel\"] | contains(\"FR\", \"FL\"))
                    ] |
                    sort_by(.info.props[\"audio.channel\"]) |
                    .[].id
                " | xargs` # Merge to one line

                # As channels were sorted, $1 is FR and $2 is FL
                flId=`echo "$ids" | awk '{print $1}'`
                frId=`echo "$ids" | awk '{print $2}'`

                # Connect new node to virtmic
                pw-link $flId $virtmicPortFlId
                pw-link $frId $virtmicPortFrId
            ) done
    } &
fi

wait

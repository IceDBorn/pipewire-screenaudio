#!/usr/bin/env bash

# This script finds and connects targetNodeSerial to virtual mic

virtmicPortFlId=$1
virtmicPortFrId=$2
targetNodeSerial=$3

source $PROJECT_ROOT/connector/util.sh

exec 2>>`UtilGetLogPathForFile $(basename $0)`

EXCLUDED_TARGETS='"AudioCallbackDriver"'

fullDumpFile=`UtilGetTempFile`
streamsFile=`UtilGetTempFile`
portsFile=`UtilGetTempFile`

function monitor-nodes() {
    tail -f /dev/null | pw-cli -m | grep --line-buffered -v 'pipewire.sec.label = "hex:'
}

set -e

# Cleanup on exit
trap "rm $fullDumpFile $streamsFile $portsFile; trap - SIGTERM && kill -- -$$" SIGTERM EXIT

pw-dump | jq -s "flatten(1)" > $fullDumpFile
UtilLog "[connect-and-monitor.sh] [Got Full Dump] File: $fullDumpFile"

# Get streams from $fullDumpFile
cat "$fullDumpFile" | jq -c '[ .[] | select(.info.props["media.class"] == "Stream/Output/Audio") ]' > $streamsFile
UtilLog "[connect-and-monitor.sh] [Got Streams] File: $streamsFile"

# Get output ports of streams from $fullDumpFile
streamIds=`cat "$streamsFile" | jq -c '.[].id' | paste -sd ','`
UtilLog "[connect-and-monitor.sh] [Got Stream IDs] IDs: $streamIds"

if [[ -n "$streamIds" ]]; then
    cat "$fullDumpFile" | jq -c "[ .[] | select(.type == \"PipeWire:Interface:Port\") | select(.info.direction == \"output\") | select(.info.props[\"node.id\"] | contains($streamIds)) ]" > $portsFile
    UtilLog "[connect-and-monitor.sh] [Got Ports] File: $portsFile"
else
    portsFile=/dev/null
fi

if [[ ! "$targetNodeSerial" -eq "-1" ]]; then
    UtilLog "[connect-and-monitor.sh] [Entering Single Node Mode] Serial: $targetNodeSerial"

    # Get target node id from $streamsFile
    targetNodeId=`cat "$streamsFile" | jq -c "[ .[] | select(.info.props[\"object.serial\"] == $targetNodeSerial) ][0].id"`
    UtilLog "[connect-and-monitor.sh] [Got Node ID] ID: $targetNodeId"

    # Get target node ports ids from $portsFile
    targetPortsFile=`UtilGetTempFile`
    cat "$portsFile" | jq -c "[ .[] | select(.info.props[\"node.id\"] == $targetNodeId) ]" > $targetPortsFile
    targetPortFlId=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][0].id"`
    targetPortFrId=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][0].id"`
    rm $targetPortsFile
    UtilLog "[connect-and-monitor.sh] [Got Ports IDs] FL ID: $targetPortFlId, FR ID: $targetPortFrId"

    # Connect target to virtmic
    pw-link $targetPortFlId $virtmicPortFlId
    pw-link $targetPortFrId $virtmicPortFrId
    UtilLog "[connect-and-monitor.sh] [Linked Ports] $targetPortFlId -> $virtmicPortFlId, $targetPortFrId -> $virtmicPortFrId"
else
    UtilLog "[connect-and-monitor.sh] [Entering All Desktop Audio Mode] Serial: $targetNodeSerial"

    # Get target nodes ids to connect from $streamsFile
    targetNodesIds=`cat $streamsFile | jq -c "[ .[] | select(.info.props[\"media.name\"] | contains($EXCLUDED_TARGETS) | not) ][].id" | paste -sd ','`
    UtilLog "[connect-and-monitor.sh] [Got Nodes IDs] IDs: $targetNodesIds"

    if [[ ! "$targetNodesIds" -eq "" ]]; then
        # Get target nodes ports ids from $portsFile
        targetPortsFile=`UtilGetTempFile`
        cat "$portsFile" | jq -c "[ .[] | select(.info.props[\"node.id\"] | contains($targetNodesIds)) ]" > $targetPortsFile
        targetPortsFlIds=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][].id"`
        targetPortsFrIds=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][].id"`
        rm $targetPortsFile
        UtilLog "[connect-and-monitor.sh] [Got Ports IDs] FL IDs: $targetPortsFlIds, FR IDs: $targetPortsFrIds"

        # Connect targets to virtmic
        echo "$targetPortsFlIds" | while read -r id; do pw-link $id $virtmicPortFlId; done
        echo "$targetPortsFrIds" | while read -r id; do pw-link $id $virtmicPortFrId; done
        UtilLog "[connect-and-monitor.sh] [Linked Ports] $targetPortsFlIds -> $virtmicPortFlId, $targetPortsFrIds -> $virtmicPortFrId"
    fi

    # Watch for new nodes to connect
    monitor-nodes | {
        stdbuf -o0 awk '/remote 0 added global/ && /Node/' |
            grep --line-buffered -oP 'id \K\d+' |
            while read -r id; do (
                UtilLog "[connect-and-monitor.sh] [Got New Node ID] ID: $id"

                # Skip excluded targets and targets with wrong class
                pw-dump "$id" | jq --exit-status -c "
                    [
                        .[] |
                        select(.id == $id) |
                        select(.info.props[\"media.name\"] | contains($EXCLUDED_TARGETS) | not) |
                        select(.info.props[\"media.class\"] == \"Stream/Output/Audio\" )
                    ][0].id
                " >/dev/null || {
                    UtilLog "[connect-and-monitor.sh] [Skipped Node ID] ID: $id"
                    exit 0
                }

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

                # If the node has been removed, ports will be empty. Skip node
                [[ -z "$flId" ]] || [[ -z "$frId" ]] && {
                    UtilLog "[connect-and-monitor.sh] [Skipped Node ID] ID: $id. Reason: Could not find all ports"
                    exit 0
                }

                UtilLog "[connect-and-monitor.sh] [Got Ports IDs] FL IDs: $flId, FR IDs: $frId"

                linkLog="[connect-and-monitor.sh] [Linking Ports]"

                # Connect new node to virtmic
                # Link FL
                linkLog="$linkLog ($flId -> $virtmicPortFlId"
                pw-link $flId $virtmicPortFlId && {
                    linkLog="$linkLog Success)"
                } || {
                    linkLog="$linkLog Fail)"
                }

                # Link FR
                linkLog="$linkLog ($frId -> $virtmicPortFrId"
                pw-link $frId $virtmicPortFrId && {
                    linkLog="$linkLog Success)"
                } || {
                    linkLog="$linkLog Fail)"
                }

                UtilLog "$linkLog"

            ) done
    } &
fi

wait

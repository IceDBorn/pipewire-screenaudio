#!/usr/bin/env bash

# This script finds and connects targetNodeSerial to virtual mic

virtmicPortFlId=$1
virtmicPortFrId=$2
targetNodeSerial=$3

source $PROJECT_ROOT/connector/util.sh

exec 2>>`UtilGetLogPathForFile $(basename $0)`

fullDumpFile=`UtilGetTempFile`
streamsFile=`UtilGetTempFile`
portsFile=`UtilGetTempFile`

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

cat "$fullDumpFile" | jq -c "[ .[] | select(.type == \"PipeWire:Interface:Port\") | select(.info.direction == \"output\") | select(.info.props[\"node.id\"] | contains($streamIds)) ]" > $portsFile
UtilLog "[connect-and-monitor.sh] [Got Ports] File: $portsFile"

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
    # Watch for new nodes to connect
    ./monitor-new-nodes.lua | {
        while read -r flId frId; do (
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

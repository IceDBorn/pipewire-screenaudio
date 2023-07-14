#!/usr/bin/env bash

targetNodeSerial="$1"
virtmicNode='pipewire-screenaudio'

EXCLUDED_TARGETS='"AudioCallbackDriver"'

set -e

# Get all nodes to check if $virtmicNode exists, and create it
pw-dump |
    jq --exit-status -c "[ .[] | select(.info.props[\"node.name\"] == \"$virtmicNode\") ][0]" >/dev/null ||
    pw-cli create-node adapter "{ factory.name=support.null-audio-sink node.name=$virtmicNode media.class=Audio/Source/Virtual object.linger=1 audio.position=[FL,FR] }"

# === Collect required data from PipeWire === #

# Get all nodes again for further processing
fullDumpFile=`mktemp`
pw-dump > $fullDumpFile

# Get id and ports of $virtmicNode
virtmicId=`cat "$fullDumpFile" | jq -c "[ .[] | select(.info.props[\"node.name\"] == \"$virtmicNode\") ][0].id"`
virtmicPortsFile=`mktemp`
cat "$fullDumpFile" | jq -c "[ .[] | select(.info.direction == \"input\") | select(.info.props[\"node.id\"] == $virtmicId) ]" > $virtmicPortsFile
virtmicPortFlId=`cat "$virtmicPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][0].id"`
virtmicPortFrId=`cat "$virtmicPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][0].id"`
rm $virtmicPortsFile

# Get streams from $fullDumpFile
streamsFile=`mktemp`
cat "$fullDumpFile" | jq -c '[ .[] | select(.info.props["media.class"] == "Stream/Output/Audio") ]' > $streamsFile

# Get output ports of streams from $fullDumpFile
portsFile=`mktemp`
streamIds=`cat "$streamsFile" | jq -c '.[].id' | paste -sd ','`
cat "$fullDumpFile" | jq -c "[ .[] | select(.type == \"PipeWire:Interface:Port\") | select(.info.direction == \"output\") | select(.info.props[\"node.id\"] | contains($streamIds)) ]" > $portsFile

# === Connect the target node(s) to $virtmicNode === #

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

    # Get target nodes ports ids from $portsFile
    targetPortsFile=`mktemp`
    cat "$portsFile" | jq -c "[ .[] | select(.info.props[\"node.id\"] | contains($targetNodesIds)) ]" > $targetPortsFile
    targetPortsFlIds=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FL\") ][].id"`
    targetPortsFrIds=`cat "$targetPortsFile" | jq -c "[ .[] | select(.info.props[\"audio.channel\"] == \"FR\") ][].id"`
    rm $targetPortsFile

    # Connect targets to virtmic
    echo "$targetPortsFlIds" | while read -r id; do pw-link $id $virtmicPortFlId; done
    echo "$targetPortsFrIds" | while read -r id; do pw-link $id $virtmicPortFrId; done

    # Watch for new nodes to connect
    tail -f /dev/null | pw-cli -m |
    stdbuf -o0 awk '/remote 0 added global/ && /Node/' |
    grep --line-buffered -oP 'id \K\d+' |
    while read -r id; do
        # 1. Find the ports with node.id == $id
        # 2. Get only the FR and FL ports
        # 3. Sort by audio.channel (FR > FL)
        # 4. Return only the ids
        ids=`pw-dump | jq -c "
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
    done
fi

# Cleanup
rm $fullDumpFile
rm $streamsFile
rm $portsFile

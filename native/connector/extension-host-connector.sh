#!/usr/bin/env bash

export LC_ALL=C
export PROJECT_ROOT="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; cd .. ; pwd -P )"
source $PROJECT_ROOT/connector/util.sh

exec 2>>`UtilGetLogPathForFile $(basename $0)`

payloadLength=`UtilBinToInt`
UtilLog "[extension-host-connector.sh] [Reading Bytes] $payloadLength"

payload=`head -c "$payloadLength"`
UtilLog "[extension-host-connector.sh] [Got Payload] $payload"

$PROJECT_ROOT/connector/pipewire-screenaudio.sh $payload | UtilTextToMessage

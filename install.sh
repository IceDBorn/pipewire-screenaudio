#!/usr/bin/env bash

set -e

projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"

( cd $projectRoot/native/connector-rs ; cargo build )

mkdir -p ~/.mozilla/native-messaging-hosts
sed "s|/usr/lib/pipewire-screenaudio|$projectRoot/native|g" $projectRoot/native/native-messaging-hosts/firefox.json > ~/.mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json

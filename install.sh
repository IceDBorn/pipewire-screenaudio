#!/usr/bin/env bash

projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"

mkdir -p ~/.mozilla/native-messaging-hosts

sed "s|/usr/lib/pipewire-screenaudio|$projectRoot/native|g" $projectRoot/native/native-messaging-hosts/firefox.json > ~/.mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json


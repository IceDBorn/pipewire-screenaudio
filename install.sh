#!/usr/bin/env bash

projectRoot="$( cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P )"

mkdir -p ~/.mozilla/native-messaging-hosts
mkdir -p ~/.config/chromium/NativeMessagingHosts
mkdir -p ~/.config/google-chrome/NativeMessagingHosts
mkdir -p ~/.config/microsoft-edge/NativeMessagingHosts

sed "s|/usr/lib/pipewire-screenaudio|$projectRoot/native|g" $projectRoot/native/native-messaging-hosts/firefox.json > ~/.mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json
sed "s|/usr/lib/pipewire-screenaudio|$projectRoot/native|g" $projectRoot/native/native-messaging-hosts/chromium.json > ~/.config/chromium/NativeMessagingHosts/com.icedborn.pipewirescreenaudioconnector.json
sed "s|/usr/lib/pipewire-screenaudio|$projectRoot/native|g" $projectRoot/native/native-messaging-hosts/chromium.json > ~/.config/google-chrome/NativeMessagingHosts/com.icedborn.pipewirescreenaudioconnector.json
sed "s|/usr/lib/pipewire-screenaudio|$projectRoot/native|g" $projectRoot/native/native-messaging-hosts/chromium.json > ~/.config/microsoft-edge/NativeMessagingHosts/com.icedborn.pipewirescreenaudioconnector.json


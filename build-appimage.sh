#!/usr/bin/env bash

rm -rf ./appdir
mkdir appdir

# Copy Icon
cp ./extension/assets/icons/icon.svg ./appdir/icon.svg

cp -r ./native ./appdir/

# Write AppRun script
cat <<"EOF" > ./appdir/AppRun
#!/usr/bin/env bash
cd $APPDIR

NATIVE_MESSAGING_HOST_FILE=~/.mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json

if [[ ! -f "$NATIVE_MESSAGING_HOST_FILE" ]]; then
  mkdir -p ~/.mozilla/native-messaging-hosts

  sed "s|/usr/lib/pipewire-screenaudio/connector/pipewire-screen-audio-connector.sh|$APPIMAGE|g" ./native/native-messaging-hosts/firefox.json > ~/.mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json
else
  ./native/connector/pipewire-screen-audio-connector.sh
fi

EOF

chmod +x ./appdir/AppRun

# Write .desktop file
cat <<"EOF" > ./appdir/Pipewire-Screenaudio.desktop
[Desktop Entry]
Name=Pipewire Screenaudio
Exec=AppRun
Type=Application
Categories=Utility
Icon=icon
EOF

ARCH=x86_64 appimagetool appdir

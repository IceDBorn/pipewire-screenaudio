# <img src="./extension/assets/icons/icon.svg" width="22" alt="Logo"> Pipewire Screenaudio
Extension to passthrough Pipewire audio to WebRTC Screenshare

Based on [link-app-to-mic](https://github.com/Soundux/rohrkabel/tree/master/examples/link-app-to-mic) and [Screenshare-with-audio-on-Discord-with-Linux](https://github.com/edisionnano/Screenshare-with-audio-on-Discord-with-Linux)

## Installation
### Building from Source
#### Requirements
- cmake
- pipewire
- tl-expected
- jq
- hexdump
- gawk

### Building

```bash
git clone https://github.com/IceDBorn/pipewire-screenaudio.git
cd pipewire-screenaudio/native
bash build.sh
```

### Installing
- Edit `pipewire-screenaudio/native/native-messaging-hosts/firefox.json`, replace "path" with the full location of `pipewire-screen-audio-connector.sh`
- Rename `firefox.json` to `com.icedborn.pipewirescreenaudioconnector.json` and move it to `~/.mozilla/native-messaging-hosts`

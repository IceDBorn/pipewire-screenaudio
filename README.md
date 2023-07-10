# <img src="./extension/assets/icons/icon.svg" width="22" alt="Logo"> Pipewire Screenaudio
Extension to passthrough Pipewire audio to WebRTC Screenshare

Based on [link-app-to-mic](https://github.com/Soundux/rohrkabel/tree/master/examples/link-app-to-mic) and [Screenshare-with-audio-on-Discord-with-Linux](https://github.com/edisionnano/Screenshare-with-audio-on-Discord-with-Linux)

## Installation
### Building from Source
#### Requirements
- cmake
- gawk
- hexdump
- jq
- pipewire
- pipewire-pulse
- tl-expected 

### Building

```bash
git clone https://github.com/IceDBorn/pipewire-screenaudio.git
cd pipewire-screenaudio/native
bash build.sh
```

### Installing
- Edit `pipewire-screenaudio/native/native-messaging-hosts/firefox.json`, replace "path" with the full location of `pipewire-screenaudio/native/connector/pipewire-screen-audio-connector.sh`
- Rename `firefox.json` to `com.icedborn.pipewirescreenaudioconnector.json` and move it to `~/.mozilla/native-messaging-hosts`
- Install the extension for Firefox

## Usage
- Optional: Grant extension with access permissions to all sites
- Join a WebRTC call, click the extension icon, select an audio node and share
- Stream, your transmission should contain both audio and video
 

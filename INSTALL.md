# Installation Guide

## Dependencies

### Extension UI

- Node.js
- Yarn

### Native Connector

- Cargo

## Browser Extension

You can install the extension from your browser's extension store or build and sideload it locally.

### Published Extensions

- [Firefox Add-ons](https://addons.mozilla.org/en-US/firefox/addon/pipewire-screenaudio/)
- [Chrome Web Store](https://chromewebstore.google.com/detail/pipewire-screenaudio/cbmjbapailadjabjnjnnbfdimkbdicja)

### Local Installation

- **Firefox-based browsers:**
  1.  Go to [about:debugging](about:debugging#/runtime/this-firefox) and click "Load Temporary Add-on".
  1.  Select the `manifest.json` file inside the `extension` directory in the project root
- **Chromium-based browsers:**
  1.  Go to [chrome://extensions](chrome://extensions) and click "Load unpacked".
  1.  Select the `extension` directory containing the `manifest.json` file in the project root

> **Note:** For Chromium-based browsers, after loading the extension, copy its ID and provide it to the install script when prompted.

## Native Messaging Hosts

The extension communicates with a native connector for pipewire management. For security reasons, browsers require explicit configuration specifying which extensions can interact with which binaries.

Configuration paths vary by browser. Some common locations are:

### Firefox-based Browsers

- `~/.mozilla/native-messaging-hosts` or `~/.config/mozilla/firefox/native-messaging-hosts` (Firefox)
- `~/.librewolf/native-messaging-hosts` (Librewolf)

### Chromium-based Browsers

- `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts` (Brave)
- `~/.config/chromium/NativeMessagingHosts` (Chromium)
- `~/.config/net.imput.helium/NativeMessagingHosts` (Helium)
- `~/.config/google-chrome/NativeMessagingHosts` (Google Chrome)

## Native Connector

- The native connector can be built from source or installed via your package manager
- The typical binary path is: `/usr/lib/pipewire-screenaudio/connector/connector-rs`

If you build the connector yourself, do not move or delete the project directory, as the binary resides within it. If you move the directory, rerun the install script to update the path.

## Uninstallation

- Remove the browser extension from your browser's extensions page
- Delete the `com.icedborn.pipewirescreenaudioconnector.json` manifest from your browser's native messaging hosts directory
- Remove the native connector binary if built from source

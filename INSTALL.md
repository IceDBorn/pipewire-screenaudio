# Installing pipewire-screenaudio

## Dependencies

#### React

- Node.js
- Yarn

#### Rust

- Cargo

## Extension

The extension can either be installed from the browser's store, or built and sideloaded.

Our current published extensions are:

- Firefox Add Ons: https://addons.mozilla.org/en-US/firefox/addon/pipewire-screenaudio/
- Chrome Web Store: TBD

## Native Messaging Hosts

The extension has to communicate with a native connector, for Pipewire management. For security reasons, browsers require explicit configuration on which extensions can communicate with which binaries.

Each browser has each own path for this configuration. Some of the known paths are:

#### Firefox Based

- `~/.mozilla/native-messaging-hosts` or `~/.config/mozilla/firefox/native-messaging-hosts` for Firefox
- `~/.librewolf/native-messaging-hosts` for Librewolf

#### Chromium Based

- `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts` for Brave
- `~/.config/chromium/NativeMessagingHosts` for Chromium
- `~/.config/net.imput.helium/NativeMessagingHosts` for Helium
- `~/.config/google-chrome/NativeMessagingHosts` for Google Chrome

## Native Connector

The native connector can either be built, or installed from your package manager. In case it's installed from the package manager, the path of the binary should be provided.

The typical path of the binary should be `/usr/lib/pipewire-screenaudio/connector/connector-rs`

If you opt to build it yourself, make sure to not delete the project, as the binary resides within it.

# <img src="./extension/assets/icons/icon.svg" width="22" alt="Logo"> Pipewire Screenaudio

Extension to passthrough Pipewire audio to WebRTC Screenshare

Based on [virtual-mic](https://github.com/Curve/rohrkabel/tree/master/examples/virtual-mic) and [Screenshare-with-audio-on-Discord-with-Linux](https://github.com/edisionnano/Screenshare-with-audio-on-Discord-with-Linux)

## Communication

You can find us on [Matrix](https://matrix.to/#/#pipewire-screenaudio:matrix.org)

## Installation

### Packages

[![AUR](https://img.shields.io/aur/version/pipewire-screenaudio?style=for-the-badge)](https://aur.archlinux.org/packages/pipewire-screenaudio)
[![AUR](https://img.shields.io/aur/version/pipewire-screenaudio-git?style=for-the-badge)](https://aur.archlinux.org/packages/pipewire-screenaudio-git)

#### NixOS Flakes

```nix
# flake.nix

{
  inputs.pipewire-screenaudio.url = "github:IceDBorn/pipewire-screenaudio";
  # ...

  outputs = {nixpkgs, pipewire-screenaudio, ...} @ inputs: {
    nixosConfigurations.HOSTNAME = nixpkgs.lib.nixosSystem {
      specialArgs = { inherit inputs; }; # this is the important part
      modules = [
        ./configuration.nix
      ];
    };
  }
}

# configuration.nix

{inputs, pkgs, ...}: {
  environment.systemPackages = with pkgs; [
    (firefox.override { nativeMessagingHosts = [ inputs.pipewire-screenaudio.packages.${pkgs.system}.default ]; })
    # ...
  ];
}
```

### Installing from Source

#### Requirements

- gawk
- hexdump
- jq
- pipewire

```bash
git clone https://github.com/IceDBorn/pipewire-screenaudio.git
cd pipewire-screenaudio
bash install.sh
```

## Usage

- #### Via the extension

  - Install the [extension](https://addons.mozilla.org/firefox/addon/pipewire-screenaudio)
  - Optional: Grant extension with access permissions to all sites
  - Join a WebRTC call, click the extension icon, select an audio node and share
  - Stream, your transmission should contain both audio and video

- #### Via the CLI

  - **Description:** It's used to manually call the commands that are normally called by the extension. It is meant for troubleshooting, but it could be used for integrating the connector with other apps.
  - **Usage:**
    ```bash
    bash native/connector/cli.sh COMMAND ARGUMENTS
    ```
  - **Example:**
    ```bash
    bash native/connector/cli.sh GetNodes
    ```
    ```bash
    bash native/connector/cli.sh SetSharingNode '{ "micId": 100, "node": 200 }'
    ```

## Known Problems

- You can't stream firefox WebRTC calls at all while using `All Desktop Audio`, they are excluded by default

### resistFingerprinting

- privacy.resistFingerprinting (enabled by default in LibreWolf, arkenfox user.js, etc.) breaks the extension. Either disable the preference or add any domains you wish to use Pipewire Screenaudio with to `privacy.resistFingerprinting.exemptedDomains` in `about:config`

### Audio pitching

- This bug exclusively impacts Firefox versions predating 120

## Planned Features

- Multiple nodes selection
- More customization options (node matching, watcher behavior etc.)
- Chromium support

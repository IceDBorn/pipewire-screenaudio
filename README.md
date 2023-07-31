# <img src="./extension/assets/icons/icon.svg" width="22" alt="Logo"> Pipewire Screenaudio
Extension to passthrough Pipewire audio to WebRTC Screenshare

Based on [link-app-to-mic](https://github.com/Soundux/rohrkabel/tree/master/examples/link-app-to-mic) and [Screenshare-with-audio-on-Discord-with-Linux](https://github.com/edisionnano/Screenshare-with-audio-on-Discord-with-Linux)

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
    (firefox.override { extraNativeMessagingHosts = [ inputs.pipewire-screenaudio.packages.${pkgs.system}.default ]; })
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
- psmisc

```bash
git clone https://github.com/IceDBorn/pipewire-screenaudio.git
cd pipewire-screenaudio
bash install.sh
```
## Usage
- Optional: Grant extension with access permissions to all sites
- Join a WebRTC call, click the extension icon, select an audio node and share
- Stream, your transmission should contain both audio and video
 
## Known Problems
- There is no way to change the audio node you're sharing while streaming
- You can't stream firefox WebRTC calls at all while using `All Desktop Audio`, they are excluded by default
### resistFingerprinting
- privacy.resistFingerprinting (enabled by default in LibreWolf, arkenfox user.js, etc.) breaks the extension. Either disable the preference or add any domains you wish to use Pipewire Screenaudio with to `privacy.resistFingerprinting.exemptedDomains` in `about:config`
### Audio pitching
> Note: A Mozilla employee responded and they are currently working on a fix
- Firefox recently implemented a feature for spawning WebRTC audio nodes, and while it works, it has a lot of pitching and de-sync issues. We created a [bug report](https://bugzilla.mozilla.org/show_bug.cgi?id=1844181) on bugzilla and we're hoping for the best!


## Planned Features
- Multiple nodes selection
- Change audio node while streaming
- More customization options (node matching, watcher behavior etc.)
- Chromium support

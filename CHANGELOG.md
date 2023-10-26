# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.4 - 2023-10-26

![Father Gascoigne](https://s6.gifyu.com/images/S8sjU.gif)

### Fixed

- Flake installation
- Enqueued actions causing the nodes loop to stop (#61) @jim3692
- micId being "null" instead of null, causing errors
- setSelectedNode(null) always failing (#70)
- Nodes loop being stopped after sharing (#67)
- Multiple additions of dropdown listeners (#68)
- Crash when starting "All Desktop Audio" without nodes (#69) @alansartorio

## 0.3.3 - 2023-10-22

### Added

- Logging (#30) @jim3692 @alansartorio

### Fixed

- Extension reporting falsely pipewire-screenaudio is running (#37) @jim3692
- Discord workaround not working when another session is open (#44) @alansartorio
- Race condition upon linking ports (#48) @alansartorio
- Race condition upon setting nodes to share (#49) @jim3692
- Being able to hide "All Desktop Audio" (#46) @jim3692

### Changed

- Code cleanup (#29 #30) @jim3692 @alansartorio

## 0.3.2 - 2023-10-10

### Fixed

- `intToBin` not working on some Linux distros (#27) @jim3692

## 0.3.1 - 2023-08-09

### Fixed

- Discord screen-sharing not working if the user does not select a video source within 5 seconds (#20) @alansartorio

## 0.3.0 - 2023-08-08

### Added

- Information about Matrix
- Note about bugzilla report
- Switching nodes while streaming @alansartorio

### Fixed

- All Desktop Audio not working sometimes @alansartorio
- Duplicated audio occurring on Discord + Wayland @alansartorio 😱

## 0.2.1 - 2023-07-20

### Added

- NixOS flakes support and installation instructions

### Changed

- License to GPL3
- Extension icon @illuminor 💘

## 0.2.0 - 2023-07-19

We reworked the whole extension to only use bash and pipewire. This was made possible by the great work of @jim3692 💋

### Added

- Install script for the native part of the extension
- Version checks

### Changed

- Port the native part to pure bash

### Removed

- C++ and its dependencies

## 0.1.1 - 2023-07-18

### Fixed

- The injector script not applying in iframes

## 0.1.0 - 2023-07-12

### Added

- "All Desktop Audio" support @alansartorio

## 0.0.1 - 2023-07-10

Initial release

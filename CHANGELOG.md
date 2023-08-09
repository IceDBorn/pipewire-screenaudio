# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

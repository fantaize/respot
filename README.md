<div align="center" style="text-align:center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="doc/images/logo_text_dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="doc/images/logo_text_light.svg">
    <img alt="respot logo" height="128" src="doc/images/logo_text_light.svg">
  </picture>
  <h3>An ncurses Spotify client written in Rust using librespot</h3>

  <img alt="respot search tab" src="doc/images/screenshot.png">
</div>

respot is an ncurses Spotify client written in Rust using librespot. It is heavily inspired by
ncurses MPD clients, such as [ncmpc](https://musicpd.org/clients/ncmpc/). My motivation was to
provide a simple and resource friendly alternative to the official client as well as to support
platforms that currently don't have a Spotify client, such as the \*BSDs.

respot only works with a Spotify premium account as it offers features that are not available for
free accounts.

## Features
- Support for tracks, albums, playlists, genres, searching...
- Small [resource footprint](doc/resource_footprint.md)
- Support for a lot of platforms
- Vim keybindings out of the box
- IPC socket for remote control

## Installation
respot is available on macOS (Homebrew), Windows (Scoop, WinGet), Linux (native package, Flathub and
Snapcraft) and the BSD's. Detailed installation instructions for each platform can be found
[here](/doc/users.md).

## Configuration
A configuration file can be provided. The default location is `~/.config/respot`. Detailed
configuration information can be found [here](/doc/users.md#configuration).

## Building
Building respot requires a working [Rust installation](https://www.rust-lang.org/tools/install) and
a Python 3 installation. To compile respot, run `cargo build`. For detailed instructions on building
respot, there is more information [here](/doc/developers.md).

## Packaging
Information about provided files, how to generate some of them and current package status accross
platforms can be found [here](/doc/package_maintainers.md).

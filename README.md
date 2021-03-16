# obs-openvr

OpenVR capture plugin for [`obs-studio`](https://github.com/obsproject/obs-studio) on Linux. Similar to [OBS-OpenVR-Input-Plugin](https://github.com/baffler/OBS-OpenVR-Input-Plugin) on Windows.

# Building & Installation

## Packages

If you're on [Arch Linux](https://archlinux.org) or one of it's family of distributions that can use the [AUR](https://aur.archlinux.org), then `obs-openvr` is available as [`obs-openvr-git`](https://aur.archlinux.org/packages/obs-openvr-git/).

## Building

`obs-openvr` is built with [`cargo`](https://crates.io), and requires the following dependent libraries.

* `libobs`
* `glfw`
* `openvr`

To build, as with any `cargo` crate, just do the following.

```bash
cargo build --release
```

The output binary will then be in `target/release/libobs_openvr.so`.

## Installation

There are 3 options for installation.

1. Installation from the [AUR](https://aur.archlinux.org) package (see above)
2. After building, install as symlink to target directory (recommended for development). `./install-link.sh release`
3. After building, install directly (recommended for normal installations). `install -Dm 0644 -T target/release/libobs_openvr.so ~/.config/obs-studio/plugins/obs-openvr/bin/64bit/libobs-openvr.so`

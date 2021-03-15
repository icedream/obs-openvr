# obs-openvr

OpenVR capture plugin for [`obs-studio`](https://github.com/obsproject/obs-studio) on Linux. Similar to [OBS-OpenVR-Input-Plugin](https://github.com/baffler/OBS-OpenVR-Input-Plugin) on Windows.

# Building & Installation

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

There are 2 options for installation.

1. Install as symlink to target directory (recommended for development). `./install-link.sh release`
2. Install directly (recommended for normal installations). `install -Dm 0644 -T target/release/libobs_openvr.so ~/.config/obs-studio/plugins/obs-openvr/bin/64bit/libobs-openvr.so`

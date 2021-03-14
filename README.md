# obs-openvr

OpenVR capture plugin for [`obs-studio`](https://github.com/obsproject/obs-studio) on Linux. Similar to [OBS-OpenVR-Input-Plugin](https://github.com/baffler/OBS-OpenVR-Input-Plugin) on Windows.

This plugin is **not** ready for prime-time yet. The checklist of remaining items for it to be ready can be found below.

- [x] Find a way to derive size of textures given by `IVRCompositor`, so get rid of the hard-coded sizes
- [ ] Implement `update()` method from OBS source API to actually obey properties set by user

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

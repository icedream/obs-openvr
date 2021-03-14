# obs-openvr

OpenVR capture plugin for [`obs-studio`](https://github.com/obsproject/obs-studio) on Linux. Similar to [OBS-OpenVR-Input-Plugin](https://github.com/baffler/OBS-OpenVR-Input-Plugin) on Windows.

This plugin is **not** ready for prime-time yet. The checklist of remaining items for it to be ready can be found below.

- [x] Find a way to derive size of textures given by `IVRCompositor`, so get rid of the hard-coded sizes
- [ ] Implement `update()` method from OBS source API to actually obey properties set by user

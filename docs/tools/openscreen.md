# OpenScreen

[OpenScreen](https://getopenscreen.com) is available through the `openscreen` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openscreen` | Silent user installer into vx store, x64 | AppImage, x64 | App ZIP, x64 / ARM64 |

```bash
vx openscreen --help
```

Official distributions and the headless help command are supported from version 2.0. Windows uses a silent per-user installer scoped to the vx store.

Linux extracts the AppImage into the managed store and launches its `AppRun`, so launching does not require a FUSE mount. Extraction failures stop installation.

The host retains its upstream license. Installing the application does not install or connect a DCC-MCP adapter. Resolve the executable with `vx where openscreen` and follow the adapter installation contract.

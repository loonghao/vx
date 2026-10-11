# OBS Studio

[OBS Studio](https://obsproject.com) is available through the `obs` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `obs` | Portable ZIP, x64 / ARM64 | Debian/Ubuntu APT package, x64 / ARM64 | Homebrew cask / installed app, x64 / ARM64 |

```bash
vx obs --version
```

Windows ARM64 packages require OBS 32 or newer. On Debian/Ubuntu, Linux installation uses `obs-studio` from the configured APT repositories and requires root or sudo access. The [Ubuntu archive](https://packages.ubuntu.com/noble/obs-studio) provides a distribution package; vx does not add the OBS PPA described in the [upstream Linux guide](https://obsproject.com/kb/linux-installation). macOS uses the `obs` Homebrew cask or an existing application. System package managers select their available release and do not guarantee the version requested in vx.

The application remains subject to its upstream license. Installing it does not install or connect a DCC-MCP adapter. Resolve its executable with `vx where obs` and follow the DCC-MCP adapter installation instructions.

# GIMP

[GIMP](https://www.gimp.org) is available through the `gimp` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `gimp` | WinGet / Chocolatey | APT / DNF / pacman | Homebrew cask / installed app |

```bash
vx gimp --version
```

Installation resolves the `system` version and uses the OS package manager. The manager chooses the actual application release; this Provider does not pin an upstream GIMP version. Linux installation may require root or sudo privileges.

Windows discovery covers the default machine and user installation directories (`Program Files` and `%LOCALAPPDATA%\Programs`). It prefers GIMP 3 console executables for command output, including the current `gimp-console-3.exe` alias and the 3.2/3.0 names, then checks GUI executables. Existing GIMP 2.10 installations are also recognized. These locations follow the [official installer](https://github.com/GNOME/gimp/blob/GIMP_3_2_6/build/windows/installer/gimp-setup.iss) and [WinGet's user and machine scopes](https://github.com/microsoft/winget-pkgs/blob/master/manifests/g/GIMP/GIMP/3/3.2.6.0/GIMP.GIMP.3.installer.yaml).

The [DCC-MCP adapter requires GIMP 3.x](https://github.com/dcc-mcp/dcc-mcp-gimp/blob/main/install.md#requirements). Check the package manager's application version before installing the adapter; GIMP 2.x is unsupported.

The host retains its upstream license. Installing the application does not install or connect a DCC-MCP adapter. Resolve the executable with `vx where gimp` and follow the adapter installation contract.

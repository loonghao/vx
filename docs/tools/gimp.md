# Gimp

[Gimp](https://www.gimp.org) is available through the `gimp` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `gimp` | WinGet, GIMP 3 | APT / DNF / pacman | Homebrew cask / installed app |

```bash
vx gimp --version
```

Installation uses the OS package manager; an exact vx version pin may differ from the package manager release.

The [DCC-MCP adapter requires GIMP 3.x](https://github.com/dcc-mcp/dcc-mcp-gimp/blob/main/install.md#requirements). Check the package manager's application version before installing the adapter; GIMP 2.x is unsupported.

The host retains its upstream license. Installing the application does not install or connect a DCC-MCP adapter. Resolve the executable with `vx where gimp` and follow the adapter installation contract.

# Gimp

通过 `gimp` Provider 使用 [Gimp](https://www.gimp.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `gimp` | WinGet / Chocolatey | APT / DNF / pacman | Homebrew cask / 已安装应用 |

```bash
vx gimp --version
```

安装解析为 `system` 版本，由操作系统包管理器选择实际应用版本；此 Provider 不锁定 GIMP 上游版本。Linux 安装可能需要 root 或 sudo 权限。

[DCC-MCP 适配器要求 GIMP 3.x](https://github.com/dcc-mcp/dcc-mcp-gimp/blob/main/install.md#requirements)。安装适配器前需确认系统包管理器提供的应用版本，不支持 GIMP 2.x。

应用保留其上游许可证。安装应用后，需要单独安装并连接 DCC-MCP 适配器。使用 `vx where gimp` 获取可执行文件路径，再遵循适配器的安装契约。

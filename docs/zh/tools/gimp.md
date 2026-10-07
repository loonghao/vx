# GIMP

通过 `gimp` Provider 使用 [GIMP](https://www.gimp.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `gimp` | WinGet / Chocolatey | APT / DNF / pacman | Homebrew cask / 已安装应用 |

```bash
vx gimp --version
```

安装解析为 `system` 版本，由操作系统包管理器选择实际应用版本；此 Provider 不锁定 GIMP 上游版本。Linux 安装可能需要 root 或 sudo 权限。

Windows 会检查机器级和用户级的默认安装目录（`Program Files` 和 `%LOCALAPPDATA%\Programs`），优先使用 GIMP 3 的控制台程序获取命令输出，覆盖当前的 `gimp-console-3.exe` 别名和 3.2、3.0 文件名，再尝试图形界面程序。也能识别已安装的 GIMP 2.10。这些目录来自[官方安装器](https://github.com/GNOME/gimp/blob/GIMP_3_2_6/build/windows/installer/gimp-setup.iss)及 [WinGet 的用户级和机器级安装清单](https://github.com/microsoft/winget-pkgs/blob/master/manifests/g/GIMP/GIMP/3/3.2.6.0/GIMP.GIMP.3.installer.yaml)。

[DCC-MCP 适配器要求 GIMP 3.x](https://github.com/dcc-mcp/dcc-mcp-gimp/blob/main/install.md#requirements)。安装适配器前需确认系统包管理器提供的应用版本，不支持 GIMP 2.x。

应用保留其上游许可证。安装应用后，需要单独安装并连接 DCC-MCP 适配器。使用 `vx where gimp` 获取可执行文件路径，再遵循适配器的安装契约。

# Gimp

通过 `gimp` Provider 使用 [Gimp](https://www.gimp.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `gimp` | WinGet，GIMP 3 | APT / DNF / pacman | Homebrew cask / 已安装应用 |

```bash
vx gimp --version
```

安装使用操作系统包管理器，其实际版本可能与 vx 指定的版本不同。

应用保留其上游许可证。安装应用后，需要单独安装并连接 DCC-MCP 适配器。使用 `vx where gimp` 获取可执行文件路径，再遵循适配器的安装契约。

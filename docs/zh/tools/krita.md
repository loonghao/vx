# Krita

通过 `krita` Provider 使用 [Krita](https://krita.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `krita` | 便携 ZIP，x64 | AppImage，x64 | Homebrew cask / 已安装的应用 |

```bash
vx krita --version
```

Windows 使用便携包中的 `krita.com` 控制台入口。Linux AppImage 需要兼容的系统库和 AppImage/FUSE 支持。macOS 使用 Homebrew cask 或已有的应用安装。

Krita 在处理 `--version` 前就会初始化 Qt，因此请在图形会话中执行示例。Provider 自动检查仅验证安装后的可执行文件存在。Krita 的实际启动和 DCC-MCP 适配器连接，需要在真实图形会话中单独验收。

Krita 保留其上游许可证。使用 `vx where krita` 获取可执行文件路径，再遵循 DCC-MCP 适配器的安装契约。安装 Krita 后，仍需单独安装并连接适配器。

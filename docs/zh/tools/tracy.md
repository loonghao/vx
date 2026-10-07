# Tracy

通过 `tracy` Provider 使用 [Tracy](https://github.com/wolfpld/tracy)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `tracy` | 性能分析器 ZIP，x64 | 包含性能分析器 AppImage 的 ZIP，x64 | 性能分析器 ZIP，ARM64 |

```bash
vx tracy --help
```

指定版本发行包从 0.14.1 起受支持。性能分析器的 `--help` 命令会输出帮助并退出，不会打开窗口。Linux 仍需兼容的系统库及 AppImage/FUSE 支持。此 Provider 不支持 macOS x64 发行包。

`tracy-capture`、`tracy-csvexport` 和 `tracy-update` Runtime 随性能分析器一起提供。自动安装检查会确认这些可执行文件存在；采集和处理跟踪数据仍需使用真实分析目标及跟踪文件验证。使用 `vx where tracy-capture` 和 `vx where tracy-csvexport` 获取适配器所需的可执行文件路径。

应用继续遵循上游许可证。安装应用不会自动安装或连接 DCC-MCP 适配器。请按 [DCC-MCP 集成指南](../guide/dcc-mcp.md)完成适配器配置。

# Tracy

通过 `tracy` Provider 使用 [Tracy](https://github.com/wolfpld/tracy)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `tracy` | 性能分析器 ZIP，x64 | 包含性能分析器 AppImage 的 ZIP，x64 | 性能分析器 ZIP，ARM64 |

```bash
vx tracy --help
vx tracy-csvexport --version
```

指定版本发行包从 0.14.1 起受支持。性能分析器在打开窗口前解析 `--help`，但系统加载器必须先找到所需的动态库。Linux 上包括 EGL（`libEGL.so.1`）及兼容的图形库，AppImage 还需要 AppImage/FUSE 支持。使用性能分析器 GUI 需要图形会话。此 Provider 不支持 macOS x64 发行包。

`tracy-capture`、`tracy-csvexport` 和 `tracy-update` Runtime 随性能分析器一起提供。性能分析器的自动验收会确认其可执行文件存在，安装完整性还要求三个辅助程序全部存在。这些检查验证发行包布局，不代表 GUI 启动成功。单独验收 `tracy-csvexport` 时会运行无界面的 `--version`，并要求输出版本。采集和处理跟踪数据仍需使用真实分析目标及跟踪文件验证。使用 `vx where tracy-capture` 和 `vx where tracy-csvexport` 获取适配器所需的可执行文件路径。

应用继续遵循上游许可证。安装应用不会自动安装或连接 DCC-MCP 适配器。请按 DCC-MCP 适配器安装说明完成适配器配置。

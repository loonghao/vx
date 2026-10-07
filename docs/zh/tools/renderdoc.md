# RenderDoc

通过 `renderdoc` Provider 使用 [RenderDoc](https://renderdoc.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `renderdoc`、`renderdoccmd` | 官方 ZIP，x64 / x86 | 官方 tar.gz，x64 | 不支持 |

```bash
vx renderdoccmd version
```

自动安装检查仅确认图形应用的可执行文件存在，不会启动应用。应用运行及 DCC-MCP 适配器连接仍需在真实应用环境中验证。随包提供的命令行程序还会通过 `renderdoccmd version` 检查。

`renderdoc` Runtime 启动 qrenderdoc 图形界面。随包提供的 `renderdoccmd` Runtime 使用同一安装目录，提供命令行功能；查询版本时使用 `version` 子命令。

Provider 会在运行环境中将 `DCC_MCP_RENDERDOC_CMD` 设置为随包提供的命令行程序路径。应用继续遵循上游许可证。安装应用不会自动安装或连接 DCC-MCP 适配器。使用 `vx where renderdoccmd` 获取命令行程序路径，再按 [DCC-MCP 集成指南](../guide/dcc-mcp.md)完成适配器配置。

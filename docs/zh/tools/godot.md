# Godot

通过 `godot` Provider 使用 [Godot](https://godotengine.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `godot` | 便携 ZIP，x64 / x86 / ARM64 | 便携 ZIP，x64 / x86 / ARM64 / ARMv7 | 通用应用 ZIP，x64 / ARM64 |

```bash
vx godot --headless --version
vx godot --headless --path game --editor --quit
```

Provider 安装标准编辑器发行版。Windows 使用控制台包装程序，并将编辑器可执行文件保留在同一目录。稳定版版本号可以带有或省略 `-stable` 后缀。

Godot 保留其上游许可证。使用 `vx where godot` 获取可执行文件路径，再遵循 DCC-MCP 适配器的安装契约。安装 Godot 后，仍需单独安装并连接适配器。

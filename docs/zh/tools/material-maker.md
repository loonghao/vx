# Material Maker

通过 `material-maker` Provider 使用 [Material Maker](https://www.materialmaker.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `material-maker` | 便携 ZIP，x64 | 便携 tar.gz，x64 | Homebrew cask / 已安装的应用 |

```bash
vx material-maker --headless --version
vx where material-maker
```

`material_maker` 和 `materialmaker` 是别名。无界面版本命令返回内置 Godot 引擎的版本；应用包版本以其上游发行标签为准。macOS 使用 [Homebrew cask](https://github.com/Homebrew/homebrew-cask/blob/main/Casks/m/material-maker.rb) 将上游 DMG 安装为 `/Applications/Material Maker.app`，也能发现该位置已有的安装。

Material Maker 保留其上游许可证。使用 `vx where material-maker` 获取可执行文件路径，再遵循 DCC-MCP 适配器的安装契约。安装 Material Maker 后，仍需单独安装并连接适配器。

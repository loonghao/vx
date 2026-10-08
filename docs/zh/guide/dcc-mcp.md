# DCC-MCP 生态

vx 为 [DCC-MCP 产品目录](https://github.com/dcc-mcp/dcc-mcp-core/blob/2d0c1f9b16ce7a79af3bdac4f1fea028530f912d/dcc-mcp-catalog.yml)中的 15 款免费开源软件提供宿主 Runtime。vx 管理软件安装与执行，DCC-MCP 提供适配器和类型化自动化工具。

## 支持范围

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| [`blender`](../tools/blender) | ZIP，x64 / 新版 ARM64 | tar.xz，x64 | Homebrew cask / 已安装应用 |
| [`freecad`](../tools/freecad) | 7z，x64，FreeCAD 1.x | AppImage，x64 / ARM64 | Homebrew cask / 已安装应用 |
| [`gimp`](../tools/gimp) | WinGet / Chocolatey（WinGet 选择 GIMP 3） | APT / DNF / pacman | Homebrew cask / 已安装应用 |
| [`godot`](../tools/godot) | ZIP | ZIP | 通用应用 ZIP |
| [`krita`](../tools/krita) | ZIP，x64 | AppImage，x64 | Homebrew cask / 已安装应用 |
| [`material-maker`](../tools/material-maker) | ZIP，x64 | tar.gz，x64 | Homebrew cask / 已安装应用 |
| [`openscad`](../tools/openscad) | ZIP，x64 / x86 | AppImage，x64 | Homebrew 开发快照 / 已安装应用 |
| [`openusd`](../tools/openusd) | `usd-core` Python 环境 | `usd-core` Python 环境 | `usd-core` Python 环境 |
| [`renderdoc`](../tools/renderdoc) | 官方 ZIP，x64 / x86 | 官方 tar.gz，x64 | 上游无宿主发行包 |
| [`tiled`](../tools/tiled) | MSI 提取到 vx store，x64 | AppImage，x64 | 通用应用 ZIP |
| [`comfyui`](../tools/comfyui) | 便携 7z，x64 | 源码 + vx Python 环境 | 源码 + vx Python 环境 |
| [`obs`](../tools/obs) | ZIP，x64 / ARM64 | 系统包管理器 | Homebrew cask / 已安装应用 |
| [`kdenlive`](../tools/kdenlive) | 官方独立 SFX 包，x64 | AppImage，x64 | Homebrew cask / 已安装应用 |
| [`openscreen`](../tools/openscreen) | 静默用户安装到 vx store，x64 | AppImage，x64 | 应用 ZIP，x64 / ARM64 |
| [`tracy`](../tools/tracy) | ZIP，x64 | ZIP，x64 | ZIP，ARM64 |

需要精确版本时使用便携发行包。系统包管理器可能安装不同版本。AppImage 需要兼容的宿主系统库；直接运行 AppImage 还需要 AppImage/FUSE 支持。OpenSCAD 和 OpenScreen 会在安装时提取 AppImage，再通过 `AppRun` 启动，无需挂载 FUSE。软件保留各自的 GPL/LGPL、MIT、BSD 或 OpenUSD 专用许可证。

便携包支持 OpenScreen 2.0、Tiled 1.12 及以上、Tracy 0.14.1 及以上；OBS Windows ARM64 包需要 OBS 32 及以上。新版 Tiled macOS 包需要 macOS 13 及以上，Tiled 1.12.0 支持 macOS 11。

macOS 上，OpenSCAD 解析为 `system` 版本，使用 `openscad@snapshot` Homebrew cask。开发快照由 Homebrew 选择，vx 不会锁定到 2021.01 等稳定版本。

DCC-MCP 的 GIMP 适配器要求 GIMP 3.x。安装适配器前需确认系统包管理器提供的应用版本。Windows 可执行文件发现使用 `C:` 盘上的已知默认目录。

ZIP 安装在 Linux 和 macOS 上保留普通 Unix 权限及通过校验的内部相对符号链接。Windows 会在提取完成后，将文件和目录链接物化为目标内容的副本，无需符号链接权限；悬空链接和复制循环会导致安装失败。两种路径都会拒绝越界目标，以及经过已有链接或重解析点的写入。

## 使用示例

```bash
vx blender --version
vx blender --background --python scene.py
vx freecadcmd model.py
vx openscad -o part.stl part.scad
vx install blender@4.5.3 freecad@1.1.4
vx where blender
vx where freecadcmd
```

FreeCAD、Krita、Kdenlive、RenderDoc、Tiled 和 Tracy 的 GUI 安装检查验证可执行文件；GUI 启动需要在图形会话中验收，无界面桥接与类型化适配器工具按各自契约验收。`freecadcmd` 随 Windows FreeCAD 包提供；Linux AppImage 通过 FreeCAD 应用 Runtime 提供。Material Maker 的 `--headless --version` 返回内置 Godot 引擎版本，应用版本以发行标签为准。

```toml
[tools]
blender = "4.5.3"
freecad = "1.1.4"
godot = "4.7.2-stable"
openscad = "2021.01"

[scripts]
render = "vx blender --background scene.blend --render-frame 1"
export = "vx openscad -o part.stl part.scad"
```

执行 `vx sync` 安装项目宿主，再运行 `vx run render` 或 `vx run export`。

## 接入 DCC-MCP

使用已安装的 DCC-MCP CLI 查看适配器目录及安装计划：

```bash
dcc-mcp-cli dcc-types --output json
vx where blender
dcc-mcp-cli install --dcc-type blender --dcc-path "<vx where 返回的 Blender 可执行文件>"
dcc-mcp-cli list
dcc-mcp-cli search --dcc-type blender --query "scene" --output json
```

按适配器计划完成安装后，分别验证连接、注册和类型化工具调用。vx 不会自动安装适配器或修改应用偏好设置。Material Maker 宿主已有 Provider，但其适配器在当前目录中仍等待 wheel 发布，应先查询实时目录。

无界面桥接适配器可使用以下环境变量指定 `vx where` 解析后的路径：

| 适配器 | 环境变量 | 路径来源 |
| --- | --- | --- |
| FreeCAD | `DCC_MCP_FREECAD_EXECUTABLE` | Windows：`vx where freecadcmd` |
| OpenSCAD | `DCC_MCP_OPENSCAD_EXECUTABLE` | `vx where openscad` |
| Tiled | `DCC_MCP_TILED_EXECUTABLE` | `vx where tiled` |
| RenderDoc command | `DCC_MCP_RENDERDOC_CMD` | `vx where renderdoccmd` |
| Kdenlive | `DCC_MCP_KDENLIVE_EXECUTABLE` | `vx where kdenlive` |
| Tracy capture | `DCC_MCP_TRACY_CAPTURE` | `vx where tracy-capture` |
| Tracy CSV export | `DCC_MCP_TRACY_CSVEXPORT` | `vx where tracy-csvexport` |

Linux 上使用 Kdenlive 渲染时，还需按照[适配器安装契约](https://github.com/dcc-mcp/dcc-mcp-kdenlive/blob/dfe3d5d1d8b7e0e3bdcc40f390dae658f8c3d859/install.md)配置 renderer 和 probe 路径。仅指定编辑器 AppImage 不会暴露其中的辅助可执行文件。

应用界面操作遵循 DCC-MCP 项目的 `dcc-cua` / `ui-control` 路由；首次观察或输入前需要报告 `provider=dcc-cua`、运行时版本、目标 PID 和 HWND。安装宿主本身不会建立界面绑定。

## ComfyUI

Windows x64 使用官方便携包，选择需要的 GPU 后端：

```bash
vx comfyui --cpu --listen 127.0.0.1
vx comfyui-amd --listen 127.0.0.1
vx comfyui-intel --listen 127.0.0.1
```

默认 `comfyui` 使用 NVIDIA 包，也接受 `--cpu`。启动入口调用包内 Python，以绝对路径执行 `ComfyUI/main.py`，继续转发用户参数。模型和自定义节点由 ComfyUI 管理。

Linux/macOS 使用独立源码目录，按照 [ComfyUI 官方安装说明](https://github.com/Comfy-Org/ComfyUI#manual-install-windows-linux)准备 vx 管理的环境：

```bash
vx git clone --branch v0.39.0 --depth 1 https://github.com/Comfy-Org/ComfyUI.git
cd ComfyUI
vx uv venv --python 3.12
vx uv pip install -r requirements.txt
vx uv run --no-project python main.py --cpu --listen 127.0.0.1
```

GPU 工作流需要按官方说明选择匹配的 PyTorch 后端。DCC-MCP 适配器连接已启动的 HTTP API，其安装过程不包含宿主软件或模型。

## OpenUSD

`openusd` 是 `uvx:usd-core::python` 的入口，提供包含原生 `pxr` 模块的隔离 Python 环境：

```bash
vx openusd@26.8 -c "from pxr import Usd; print(Usd.GetVersion())"
vx openusd@26.8 inspect_stage.py
```

具体平台需要有匹配 Python 版本的 wheel。该入口不包含 `usdview` 或完整 OpenUSD 构建。DCC-MCP OpenUSD 适配器要启用 native 模式，需将可选 `usd-core` 依赖安装到**适配器自身的 Python 环境**；vx 的隔离环境不会修改其他环境。

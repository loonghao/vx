# DCC-MCP ecosystem

vx manages the free and open-source hosts in the [DCC-MCP product catalog](https://github.com/dcc-mcp/dcc-mcp-core/blob/2d0c1f9b16ce7a79af3bdac4f1fea028530f912d/dcc-mcp-catalog.yml). DCC-MCP provides the adapters and typed automation tools. Installing a host with vx and connecting its adapter are separate steps.

## Available runtimes

The table describes upstream distribution formats supported by each Provider. An OS package manager may install a different release from a version pinned in vx; use portable distributions when an exact version is required. AppImages need the host's graphical libraries and compatible AppImage/FUSE support.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| [`blender`](../tools/blender) | Portable ZIP, x64 / recent ARM64 | Portable tar.xz, x64 | Homebrew cask / installed app |
| [`freecad`](../tools/freecad) | Portable 7z, x64, FreeCAD 1.x | AppImage, x64 / ARM64 | Homebrew cask / installed app |
| [`gimp`](../tools/gimp) | WinGet, GIMP 3 | APT / DNF / pacman | Homebrew cask / installed app |
| [`godot`](../tools/godot) | Portable ZIP | Portable ZIP | Universal app ZIP |
| [`krita`](../tools/krita) | Portable ZIP, x64 | AppImage, x64 | Homebrew cask / installed app |
| [`material-maker`](../tools/material-maker) | Portable ZIP, x64 | Portable tar.gz, x64 | Homebrew cask / installed app |
| [`openscad`](../tools/openscad) | Portable ZIP, x64 / x86 | AppImage, x64 | Homebrew cask / installed app |
| [`openusd`](../tools/openusd) | Native `usd-core` Python environment | Native `usd-core` Python environment | Native `usd-core` Python environment |
| [`renderdoc`](../tools/renderdoc) | Official ZIP, x64 / x86 | Official tar.gz, x64 | No upstream host distribution |
| [`tiled`](../tools/tiled) | MSI extraction into vx store, x64 | AppImage, x64 | Universal app ZIP |
| [`comfyui`](../tools/comfyui) | Portable 7z, x64 | See source environment below | See source environment below |
| [`obs`](../tools/obs) | Portable ZIP, x64 / ARM64 | System package manager | Homebrew cask / installed app |
| [`kdenlive`](../tools/kdenlive) | Official standalone SFX archive, x64 | AppImage, x64 | Homebrew cask / installed app |
| [`openscreen`](../tools/openscreen) | Silent user installer into vx store, x64 | AppImage, x64 | App ZIP, x64 / ARM64 |
| [`tracy`](../tools/tracy) | Profiler ZIP, x64 | Profiler ZIP, x64 | Profiler ZIP, ARM64 |

Upstream licensing varies: GNU GPL/LGPL, MIT, BSD and OpenUSD's Tomorrow Open Source Technology License. Provider metadata records the individual license; the original software retains its license.

Portable version support starts at OpenScreen 2.0, Tiled 1.12 and Tracy 0.14.1; OBS Windows ARM64 packages require OBS 32 or newer. Recent Tiled macOS packages require macOS 13 or newer (Tiled 1.12.0 supports macOS 11).

## Run and pin hosts

```bash
vx blender --version
vx blender --background --python scene.py
vx freecadcmd model.py
vx openscad -o part.stl part.scad
vx godot --headless --version
vx install blender@4.5.3 freecad@1.1.4
vx where blender
vx where freecadcmd
```

Installation checks for the FreeCAD, Krita, Kdenlive, RenderDoc and Tiled GUI runtimes verify their executable files. A graphical session is required to validate application startup and adapter connectivity. `freecadcmd` is bundled with the Windows FreeCAD distribution. The Linux AppImage is exposed as the FreeCAD application runtime. `material-maker --headless --version` reports its embedded Godot engine version; its application package version comes from the upstream release tag.

Project configuration can include hosts alongside ordinary development runtimes:

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

Use `vx sync` to install the selected hosts, then `vx run render` or `vx run export`.

## Connect DCC-MCP

Use an already installed `dcc-mcp-cli` to inspect the current adapter catalog and installation plan:

```bash
dcc-mcp-cli dcc-types --output json
vx where blender
dcc-mcp-cli install --dcc-type blender --dcc-path "<resolved Blender executable>"
dcc-mcp-cli list
dcc-mcp-cli search --dcc-type blender --query "scene" --output json
```

Supply the executable returned by `vx where`, and follow the adapter's installation plan. The CLI, adapter availability, application connection, and typed tool results must each be verified. vx does not automatically install adapters, modify application preferences, grant UI permissions, or register running applications.

For headless bridge adapters, set their documented environment variables to the resolved executable paths:

| Adapter | Environment variable | Resolve with |
| --- | --- | --- |
| FreeCAD | `DCC_MCP_FREECAD_EXECUTABLE` | `vx where freecadcmd` on Windows |
| OpenSCAD | `DCC_MCP_OPENSCAD_EXECUTABLE` | `vx where openscad` |
| Tiled | `DCC_MCP_TILED_EXECUTABLE` | `vx where tiled` |
| RenderDoc command | `DCC_MCP_RENDERDOC_CMD` | `vx where renderdoccmd` |
| Kdenlive | `DCC_MCP_KDENLIVE_EXECUTABLE` | `vx where kdenlive` |
| Tracy capture | `DCC_MCP_TRACY_CAPTURE` | `vx where tracy-capture` |
| Tracy CSV export | `DCC_MCP_TRACY_CSVEXPORT` | `vx where tracy-csvexport` |

Material Maker's host is supported here, while its DCC-MCP catalog currently reports that adapter auto-install is unavailable pending wheel publication. Check the live catalog before attempting adapter installation.

For Kdenlive rendering on Linux, also configure the renderer and probe paths described in the [adapter installation contract](https://github.com/dcc-mcp/dcc-mcp-kdenlive/blob/dfe3d5d1d8b7e0e3bdcc40f390dae658f8c3d859/install.md). Selecting the editor AppImage alone does not expose its internal helper executables.

Application UI control follows the DCC-MCP project's `dcc-cua` / `ui-control` route. Before any UI observation or input, bind and report `provider=dcc-cua`, the runtime version, target PID and HWND. Installing a Provider does not establish that binding.

## ComfyUI environments

On Windows x64, select the official portable backend package:

```bash
vx comfyui --cpu --listen 127.0.0.1
vx comfyui-amd --listen 127.0.0.1
vx comfyui-intel --listen 127.0.0.1
```

The default `comfyui` package is the NVIDIA portable distribution and also accepts `--cpu`. The Provider uses the package's embedded Python with an absolute path to `ComfyUI/main.py`; user arguments are forwarded unchanged. Models and custom nodes are managed by ComfyUI.

For Linux/macOS, use a separate source checkout and a vx-managed environment, following [ComfyUI's installation instructions](https://github.com/Comfy-Org/ComfyUI#manual-install-windows-linux):

```bash
vx git clone --branch v0.39.0 --depth 1 https://github.com/Comfy-Org/ComfyUI.git
cd ComfyUI
vx uv venv --python 3.12
vx uv pip install -r requirements.txt
vx uv run --no-project python main.py --cpu --listen 127.0.0.1
```

Select the upstream PyTorch backend appropriate for your GPU before running GPU workflows. The ComfyUI adapter connects to the running HTTP API; installing that adapter does not install the ComfyUI host or its models.

## OpenUSD Python SDK

`openusd` routes to `uvx:usd-core::python`. It supplies native `pxr` bindings in an isolated Python environment:

```bash
vx openusd@26.8 -c "from pxr import Usd; print(Usd.GetVersion())"
vx openusd@26.8 inspect_stage.py
```

Wheel availability depends on the Python version and platform. This runtime does not include `usdview` or the complete OpenUSD build. To enable native mode in a DCC-MCP OpenUSD adapter, install its optional `usd-core` dependency into the **same environment as that adapter**; vx's isolated environment does not modify other Python environments.

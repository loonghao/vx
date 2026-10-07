# ComfyUI

通过 `comfyui` Provider 使用 [ComfyUI](https://www.comfy.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `comfyui` | 便携 7z，x64 | 源码 + 独立 Python 环境 | 源码 + 独立 Python 环境 |

```bash
vx comfyui --cpu --listen 127.0.0.1
vx comfyui-amd --listen 127.0.0.1
vx comfyui-intel --listen 127.0.0.1
```

便携发行包支持 Windows x64。默认使用 NVIDIA 版本，也接受 `--cpu`。模型与自定义节点由 ComfyUI 管理。Linux/macOS 需要上游源码和独立的 vx Python 环境。

应用保留其上游许可证。安装应用后，需要单独安装并连接 DCC-MCP 适配器。DCC-MCP 适配器连接运行中的 ComfyUI HTTP API；请遵循适配器的安装与连接契约。

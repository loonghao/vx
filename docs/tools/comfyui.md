# ComfyUI

[ComfyUI](https://www.comfy.org) is available through the `comfyui` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `comfyui` | Portable 7z, x64 | Source checkout + Python environment | Source checkout + Python environment |

```bash
vx comfyui --cpu --listen 127.0.0.1
vx comfyui-amd --listen 127.0.0.1
vx comfyui-intel --listen 127.0.0.1
```

Portable packages support Windows x64. The default backend is NVIDIA and also accepts `--cpu`. Models and custom nodes remain managed by ComfyUI. Linux/macOS require an upstream source checkout and a separate vx-managed Python environment.

The host retains its upstream license. Installing the application does not install or connect a DCC-MCP adapter. The DCC-MCP adapter connects to the running ComfyUI HTTP API. Follow its installation and connection contract.

# OpenUSD

[OpenUSD](https://openusd.org) is available through the `openusd` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openusd` | Native `usd-core` Python environment | Native `usd-core` Python environment | Native `usd-core` Python environment |

```bash
vx openusd@26.8 -c "from pxr import Usd; print(Usd.GetVersion())"
```

This alias supplies native `pxr` bindings from the `usd-core` wheel in an isolated Python environment. It does not include `usdview` or the complete SDK. Install the optional native dependency into the adapter environment separately.

The host retains its upstream license. Installing the application does not install or connect a DCC-MCP adapter. Run Python scripts with `vx openusd` and follow the adapter installation contract for its own Python environment.

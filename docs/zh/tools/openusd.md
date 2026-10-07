# OpenUSD

通过 `openusd` Provider 使用 [OpenUSD](https://openusd.org)。

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openusd` | 原生 `usd-core` Python 环境 | 原生 `usd-core` Python 环境 | 原生 `usd-core` Python 环境 |

```bash
vx openusd@26.8 -c "from pxr import Usd; print(Usd.GetVersion())"
```

此别名在隔离的 Python 环境中提供 `usd-core` wheel 的原生 `pxr` 绑定，不包含 `usdview` 或完整 SDK。适配器需要原生功能时，需在适配器自己的环境中另行安装可选依赖。

应用保留其上游许可证。安装应用后，需要单独安装并连接 DCC-MCP 适配器。使用 `vx openusd` 执行 Python 脚本，并按照适配器的安装契约配置它自己的 Python 环境。

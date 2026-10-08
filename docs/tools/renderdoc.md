# RenderDoc

[RenderDoc](https://renderdoc.org) is available through the `renderdoc` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `renderdoc`, `renderdoccmd` | Official ZIP, x64 / x86 | Official tar.gz, x64 | Unsupported |

```bash
vx renderdoccmd version
```

Automated installation checks verify the GUI executable file without starting the application. Running the application and connecting its DCC-MCP adapter still require validation in a real application environment. The bundled command-line Runtime has a separate check: `vx test renderdoccmd` runs `renderdoccmd version`.

The `renderdoc` Runtime launches the qrenderdoc graphical application. The bundled `renderdoccmd` Runtime provides the command-line interface and shares the same installation. Its version command uses the `version` subcommand.

The Provider sets `DCC_MCP_RENDERDOC_CMD` to the bundled command-line executable in the runtime environment. The application remains subject to its upstream license. Installing it does not install or connect a DCC-MCP adapter. Resolve the command-line executable with `vx where renderdoccmd` and follow the DCC-MCP adapter installation instructions.

# vx shim

把任意 Runtime 暴露成可以直接输入的命令，之后输入 `jq --version` 即可，
不必再写 `vx jq --version`。

`vx shim` 会在 PATH 上的目录里生成一个小的包装脚本，把全部参数转发给
`vx <runtime>`，runtime 在首次调用时按需安装。这与很多人手工写进
`~/.local/bin` 的脚本是同一个东西，只不过由 vx 按平台生成并接管管理。

```bash
vx shim add jq
jq --version          # -> jq-1.8.1
```

> **RFC**：[RFC 0042 — 按平台生成的命令 shim](../../rfcs/0042-platform-command-shims.md)

## 子命令

| 子命令 | 用途 |
|---|---|
| `add` | 为某个 runtime 创建 shim |
| `list` | 列出 vx 创建的 shim（别名 `ls`） |
| `remove` | 删除 vx 创建的 shim（别名 `rm`） |
| `sync` | 按当前 `vx` 可执行文件重写全部 shim |
| `path` | 显示目标目录及其是否在 PATH 上 |

## add

```bash
vx shim add jq                       # 创建 `jq` 命令
vx shim add git@2.53.0               # 用 runtime 名承载固定版本
vx shim add jq --as jqp              # 使用不同的命令名
vx shim add jq --dir ~/.local/bin    # 指定目录（可重复）
vx shim add git --force              # 允许覆盖系统自带的 git
```

| 参数 | 说明 |
|---|---|
| `--as <NAME>` | 要创建的命令名，默认为去掉 `@version` 的 runtime 名。 |
| `--dir <DIR>` | 写入目录，可重复。默认为 vx 的 bin 目录与 `vx` 可执行文件所在目录。 |
| `-f, --force` | 覆盖并非 vx 创建的同名命令。 |

### 覆盖保护

如果名字已经解析到 PATH 上一个并非 vx 创建的二进制，`add` 会直接拒绝：

```text
$ vx shim add git
✗ 'git' already resolves to C:\Program Files\Git\mingw64\bin\git.exe on PATH.
  Re-run with --force to shadow it, or pick another name with --as.
```

可以用 `--as` 换成别的名字，或在确实需要覆盖时使用 `--force`。
vx 自己创建的 shim 永远可以直接覆盖。

## list

```bash
vx shim list
vx shim list --json      # 机器可读
```

```text
Command shims (2)
  git              -> vx git                  [ok]
    C:\Users\me\.vx\bin
  jq               -> vx jq                   [ok]
    C:\Users\me\.vx\bin
```

显示 `incomplete` 表示记录里的文件缺失，运行 `vx shim sync` 即可修复。

## remove

```bash
vx shim remove jq
```

只删除 vx 创建的文件。同名但由用户手写的包装脚本会被提示并保留。

## sync

```bash
vx shim sync
```

生成的脚本里烘焙了 `vx` 的绝对路径。升级或移动 vx 之后，`sync` 会按新位置
重写全部已注册的 shim，并补齐缺失的文件。

## path

```bash
vx shim path
```

打印每个目标目录、其是否在 PATH 上，以及补齐 PATH 的确切命令：

```text
Command shim directories
  C:\Users\me\.vx\bin      not on PATH
    Platform variants: <name>.cmd (cmd.exe, PowerShell), <name> (sh, Git Bash, MSYS2)
💡 $env:PATH = "C:\Users\me\.vx\bin;$env:PATH"
```

vx 不会修改你的 shell 配置或 Windows 注册表，是否加入 PATH 由你决定。

## 平台行为

一次 `add` 会生成当前平台需要的全部文件：

| 平台 | 文件 | 可从何处调用 |
|---|---|---|
| Windows | `jq.cmd` | cmd.exe、PowerShell（`PATHEXT` 把 `jq` 解析到 `jq.cmd`） |
| Windows | `jq` | Git Bash、MSYS2、Cygwin |
| Linux / macOS | `jq` | 所有 POSIX shell |

Windows 需要两个文件：POSIX shell 脚本对 cmd.exe 和 PowerShell 不可见，
而批处理脚本在 Git Bash 里又无法使用。Unix 终端都认 `/bin/sh`，一个脚本足够。

生成的脚本带有 `vx-shim` 标记行，并会传播被包装命令的退出码。

## 安全性

- 未加 `--force` 时，`add` 绝不覆盖没有 vx 标记的文件。
- `remove` 只删除 registry 里记录**且**带有 `vx-shim` 标记的文件。
- registry 位于 `$VX_HOME/config/command-shims.json`，遵循 `VX_HOME`。

## 相关

- [`vx global`](./global) — 全局包管理，使用同样的 shim 堆叠布局。
- [隐式包执行](./implicit-package-execution) — 无需预先安装即可运行包。

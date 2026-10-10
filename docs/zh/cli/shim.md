# vx shim

将 Runtime 或包的可执行文件暴露为普通命令，并继续由 vx 准备运行环境。

```bash
vx codex --version          # 在隔离包环境中安装并运行
vx shim add codex           # 显式创建 codex 命令入口
codex --version             # 通过 vx 使用受管理的依赖
```

`vx codex` 不会创建全局命令 shim。`vx shim add` 生成的包装脚本绑定 vx
可执行文件的绝对路径和创建时的 `VX_HOME`，因此从其他目录或 shell 调用时，
仍使用同一套受管理的环境。

公开接口是 `vx shim add/list/remove/sync/path`。创建入口必须使用 `add`；
`vx shim codex` 和不带子命令的 `vx shim` 都不是创建命令。

## 子命令

| 子命令 | 用途 |
|---|---|
| `add` | 为 Runtime 或包命令创建 shim |
| `list` | 列出已登记的 shim（别名 `ls`） |
| `remove` | 移除已登记的 shim（别名 `rm`） |
| `sync` | 按当前 vx 可执行文件刷新已登记的 shim |
| `path` | 显示默认目标目录及其 PATH 状态 |

## add

```bash
vx shim add jq
vx shim add git@2.53.0
vx shim add codex --as codex-vx
vx shim add "npm:@openai/codex::codex" --as codex-vx
vx shim add codex --dir "/absolute/path/to/shim-bin"
```

| 参数 | 说明 |
|---|---|
| `--as <NAME>` | 指定命令名，替代默认的可执行文件名。 |
| `--dir <DIR>` | 目标目录，可重复；替代默认目录。相对路径在创建时转换为绝对路径并保存。 |
| `-f, --force` | 允许创建会遮蔽 PATH 上其他同名命令的入口。 |

默认写入 `$VX_HOME/bin` 和当前 vx 可执行文件所在目录，并去除重复目录。
常规安装已将 vx 所在目录加入 PATH。自定义目录需要加入调用方的 PATH。

`add` 本身不安装或运行目标，首次调用生成的命令时可以按需安装。
`codex` 等 Provider 别名会解析为包执行请求；显式包语法可通过
`::executable` 选择与包名不同的可执行文件。创建 shim 本身不会将已安装的包
切换到另一个版本。

### 冲突保护

`add` 同时检查 PATH 上的同名命令和所有目标文件，包括 Windows 的两种包装
文件。未修改的已登记 shim 可以刷新；无关文件、用户修改过的脚本和符号链接
会被保留。发生文件冲突时，应改用其他 `--as` 名称或目标目录。

`--force` 允许 PATH 命令遮蔽，不会绕过目标文件的归属检查，也不会覆盖系统
二进制。

## list

```bash
vx shim list
vx shim list --json
```

文本输出检查已登记的文件是否与预期包装脚本一致。`incomplete` 表示文件缺失
或内容已变化；`sync` 可重建缺失文件，但不会覆盖冲突文件。
JSON 包含执行请求、launcher、`vx_home`、目录和文件路径。
包装脚本完整不代表目标包已安装，也不代表它在 PATH 上优先被找到。

## remove

```bash
vx shim remove codex
```

移除登记记录及其未修改、归属明确的包装文件。已修改的文件和无关替代文件会
被提示并保留；`--force` 不会绕过归属检查。
包仍然保持安装状态，之后运行 `vx codex` 也不会重新创建 shim。

## sync

```bash
vx shim sync
```

在当前 `VX_HOME` 中，按正在运行的 vx 可执行文件刷新已登记的 shim，保留原有
目标目录并重建缺失的文件。冲突文件会被保留并报错。旧记录若未绑定 home，
会在刷新时补充绑定。

`sync` 不安装或升级目标包，也不会为尚未登记 shim 的包创建全局命令。

## path

```bash
vx shim path
```

显示默认目标目录及其是否在当前进程的 PATH 上。自定义 `--dir` 目录可通过
`vx shim list --json` 查看。vx 会提示如何设置 PATH，但不会修改 shell
配置、持久 PATH 或 Windows 注册表。

## 显式包安装

为保持兼容，`vx install codex` 和 `vx pkg install npm:@openai/codex`
会安装包并发布其可执行文件。这些入口与 `vx shim add` 共用登记记录和受管理的
执行路径，可通过 `vx shim list/sync/remove` 管理。

`vx pkg shim-update` 会显式发布已安装包的可执行文件，并保留无关的命令 shim
和文件。仅需刷新现有登记入口时，使用 `vx shim sync`。卸载包会移除属于该包的
已登记命令入口。

旧版生成但未写入命令 shim 登记记录的包装文件会保持不变，可能在创建入口时
触发冲突。可使用其他 `--as` 名称，必要时配合私有 `--dir`；vx 不会根据文件名
或内容猜测其归属并自动接管。

## 平台行为

| 平台 | 生成的文件 | 调用方 |
|---|---|---|
| Windows | `<name>.cmd` | cmd.exe、PowerShell |
| Windows | `<name>` | Git Bash、MSYS2、Cygwin |
| Linux / macOS | `<name>` | POSIX shell |

Windows 会同时生成批处理和 POSIX 包装脚本，不生成 `.ps1` 文件。
脚本转发参数和子进程退出码。无法执行脚本的原生进程启动器应直接调用 vx，
并分别传递参数。

登记文件位于 `$VX_HOME/config/command-shims.json`。读取失败或 JSON 格式错误
会被报告，不会被当作空记录覆盖。

## 相关

- [受管理的命令 shim（英文）](../../guide/managed-command-shims.md)
- [RFC 0042 — 按平台生成的命令 shim](../../rfcs/0042-platform-command-shims.md)
- [`vx global`](./global) — 隔离包管理
- [隐式包执行](./implicit-package-execution)

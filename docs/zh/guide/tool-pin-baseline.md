# 工具版本 pin 基线

> 对「`vx.toml` 里的 `[tools]` 该怎么写」给出一个可机械校验的答案。
> 如果你用 vx 维护多个仓库，请统一采用本基线 —— 重点不是规则多精妙，而是每个仓库对同一个问题的答案一致。

## 为什么需要基线

下面每条规则都来自两类故障。

1. **两处 pin 静默冲突。** `vx.toml` 里写 `rust = "1.95"`，仓库里同时还有 `rust-toolchain.toml`，
   就有了两个真源。只有一个生效，而且没有任何提示告诉你生效的是哪个。
2. **pin 其实是浮动的。** `python = "latest"` 等于没记录任何版本 —— 昨天跑 CI 用的工具链，
    在任何地方都没有被记录下来。

## 版本写法

| 写法                | 解析结果                                   |
| ------------------- | ------------------------------------------ |
| `major.minor`       | 该 minor 线下最新的补丁版（`3.12` → `3.12.x`） |
| `major.minor.patch` | 精确版本                                   |
| `latest`            | 最新稳定版                                 |

完整语法见[版本管理](./version-management.md)。

## 三类工具

| 类别         | 定义                       | 允许写法                              | 示例                    |
| ------------ | -------------------------- | ------------------------------------- | ----------------------- |
| **运行时**   | 代码运行其上的语言运行时   | `major.minor`；精确补丁需附原因注释   | `python = "3.12"`       |
| **构建工具** | 不产出发布产物             | `latest`                              | `uv = "latest"`         |
| **产物工具** | 其版本会影响发布的产物     | 精确版本，或 `major.minor` + 原因注释 | `maturin = "1.9.6"`     |

常见归属：**运行时** —— `python` / `node` / `rust`；**构建工具** —— `uv` / `just` / `cmake` /
`prek` / `actionlint` / `sccache`；**产物工具** —— `maturin` / `cargo-llvm-cov` /
`cargo-nextest` / `msvc`。

## 规则

- **R1 —— 每个工具只有一个真源。** 生态本身已经负责 pin 时（`rust-toolchain.toml`），
  `vx.toml` 不得重复声明。
- **R2 —— 运行时禁止浮动。** 运行时类别里禁用 `latest` / `stable` / `nightly` / `*`。
- **R3 —— 精确 pin 必须写原因。** `major.minor.patch` 或产物工具的 pin 要有行尾注释，
  说明对应的回归或约束。
- **R4 —— 不重复写默认值。** 某项配置如果等于 vx 的默认值，就删掉它。
- **R5 —— 例外写在行内。** 偏离本页规则的地方在 `vx.toml` 里写一行注释，不写 wiki。

## Rust

Rust 是唯一有「原生 pin 文件」竞争的运行时，因此单列规则。vx 按以下顺序判定归属：

1. 从工作目录向上查找 `rust-toolchain.toml` / `rust-toolchain` → **归 rustup**。vx 不安装、
   不把自身 store 加到 `PATH`、绝不改动 rustup 的默认值。
2. 环境变量 `RUSTUP_TOOLCHAIN` → **归 rustup**。
3. `vx.toml` 里 `rust = "rustup-managed"` → **归 rustup**。这是「刻意不让 vx 管 Rust」的显式声明。
4. 以上都没有 → **归 vx**，装进自己的 store。即使如此，工具链也通过 `RUSTUP_TOOLCHAIN`
   注入子进程环境，绝不改写用户全局的 rustup default。

| 仓库现状                              | `vx.toml` 里该写什么                              |
| ------------------------------------- | ------------------------------------------------- |
| 有 `rust-toolchain.toml`              | 什么都不写 —— 那个文件就是 pin                   |
| 有 rustup，但没有 toolchain 文件      | `rust = "rustup-managed"`                         |
| 完全没有 rustup（裸容器、精简 CI 镜像） | `rust = "1.95"` —— `major.minor`，绝不写通道名    |

归属顺序与 `rustup-managed` 哨兵值随 rust-toolchain 归属特性的发布生效。在更老的 vx 上，
请省略 `rust` 行，只保留 `rust-toolchain.toml`。

## vx.lock —— 提交进版本库

- `vx.lock` **覆盖** `vx.toml`：解析时锁定版本优先，所以真正让 CI 可复现的是 lock 文件。
- **提交 `vx.lock`。** 每次改动 `[tools]` 都要重新生成（`vx lock`）—— 过期的 lock 会继续
  pin 旧版本，让你对 `vx.toml` 的修改看起来「没生效」。
- CI 门禁：

  ```bash
  test -f vx.lock || { echo "vx.lock missing; run 'vx lock'"; exit 1; }
  vx lock --check
  ```

  `vx lock --check` 在以下情况失败：工具缺失于 lock、lock 里的 `resolved_from` 与
  `vx.toml` 不一致、lock 里残留 `vx.toml` 已删除的工具。但 lock 文件**不存在时它返回 0**，
  所以「文件存在性」必须单独检查。
- 一份提交的 lock 可以服务所有平台：锁定的版本号与平台无关，`platform_urls` 按目标平台
  保存下载地址，`platform` 元数据只记录 lock 是在哪台机器上生成的。

## `[settings]` —— 删掉默认值

| 键                  | vx 的行为                     | 结论                              |
| ------------------- | ----------------------------- | --------------------------------- |
| `auto_install`      | 默认为 `true`                 | 除非要设成 `false`，否则删除该行  |
| `cache_duration`    | 会解析，但解析过程并不消费    | 删除 —— 无效配置                  |
| `parallel_install`  | 会解析，但解析过程并不消费    | 删除 —— 无效配置                  |

`cache_duration` 与 `parallel_install` 仍会被解析、迁移，也会被 `vx init` 写出来，但
resolver 没有任何代码读取它们，只会增加 diff 噪音。真正需要覆盖默认值时才保留
`[settings]`（例如离线 runner 上的 `auto_install = false`），并注释说明原因。

## 例外

任何偏离基线的条目都要带行尾注释：

```toml
[tools]
python = "3.14.4"   # rez 3.x 需要 3.14 的 importlib.metadata 修复；见 issue #1234
rust   = "1.90.0"   # 内置 CEF 封装的 MSRV；随封装一起升级
```

## 可机械校验的门禁

```bash
# R1：toolchain 文件之外不得重复 pin rust
if test -f rust-toolchain.toml && grep -qE '^[[:space:]]*rust[[:space:]]*=' vx.toml; then
  echo "rust is pinned twice (rust-toolchain.toml and vx.toml)"; exit 1
fi
# R2：运行时不得浮动
if grep -qE '^[[:space:]]*(python|node|rust)[[:space:]]*=[[:space:]]*"(latest|stable|nightly|\*)"' vx.toml; then
  echo "runtime pinned to a floating channel"; exit 1
fi
# R4：settings 不得重复默认值
if grep -qE '^[[:space:]]*(auto_install[[:space:]]*=[[:space:]]*true|cache_duration|parallel_install)' vx.toml; then
  echo "[settings] restates vx defaults"; exit 1
fi
# vx.lock 存在且与 vx.toml 一致
test -f vx.lock || { echo "vx.lock missing; run 'vx lock'"; exit 1; }
vx lock --check
```

R3 与 R5 是评审期规则：`major.minor.patch` 或产物工具的 pin 若没有行尾注释，评审不予通过。

## 相关文档

- [配置](./configuration.md) —— `vx.toml` 的所有段落
- [版本管理](./version-management.md) —— 完整的版本语法
- [最佳实践](./best-practices.md) —— 结合实际场景的 pin 建议

## /fix-provider-smoke

处理 provider-smoke 测试问题：根据首个参数定位 `.codebuddy/issues/<issue>.md`，读取后按步骤复现并修复。

### Usage
- `/fix-provider-smoke <issue-name>`
  - 例如：`/fix-provider-smoke provider-smoke-msvc`（对应 `.codebuddy/issues/provider-smoke-msvc.md`）

### Workflow
1) 解析 issue 名，定位 `.codebuddy/issues/<issue-name>.md`，读取上下文（平台、失败命令、日志）。
2) 复现失败：按 issue 中的命令顺序执行（如 `vx list`, `vx where`, `vx <alias> --version`）。记录新的输出/退出码。
3) 定位根因：检查 provider/runtime 映射、路径解析、安装逻辑、别名转发等。
4) 实施修复：修改代码/配置，必要时补充测试；运行相关 `cargo check/test` 或目标命令验证通过。
5) 更新 issue：追加“Resolution”章节，包含：
   - 结论（已修复/需后续）
   - 关键改动/命令
   - 验证结果（命令 + 核心输出/退出码）
6) 若仍有阻塞，注明下一步与所需信息。

### Notes
- issue 文件不存在则提示创建或检查名称。
- 保留原始日志，追加新验证日志，避免覆盖历史信息。

# VX CLI 重设计提案

## 概述

基于CLI设计最佳实践和对优秀CLI工具（cargo、npm、git、docker）的分析，本文档提出了vx CLI命令结构的重设计方案。

## 设计原则

### 1. 一致性 (Consistency)
- 统一的命名约定
- 一致的参数和选项设计
- 标准化的输出格式

### 2. 直观性 (Intuitiveness)
- 使用标准动词和名词
- 清晰的命令分组
- 符合用户期望的行为

### 3. 可发现性 (Discoverability)
- 逻辑清晰的命令层次
- 丰富的帮助信息
- 智能的错误提示和建议

### 4. 简洁性 (Simplicity)
- 避免不必要的复杂性
- 提供合理的默认值
- 支持常用操作的简写

## 当前问题分析

### 命名不一致
- `shell-init` 使用连字符，其他命令不用
- `where` 命令名不够标准（应该是 `which`）
- `fetch` 与 `list` 功能重叠

### 命令分组不清晰
- 工具管理、项目管理、环境管理混杂
- 缺少清晰的功能边界

### 缺少标准约定
- 没有遵循常见的CLI约定
- 缺少常用命令的简写形式

## 新命令结构设计

### 核心工具管理 (Tool Management)

```bash
# 安装和卸载
vx install <tool>[@version]     # 安装工具版本
vx uninstall <tool>[@version]   # 卸载工具版本 (改名from remove)
vx update [tool]                # 更新工具到最新版本

# 查询和切换
vx list [--installed|--available]  # 列出工具
vx which <tool>                 # 显示工具安装位置 (改名from where)
vx versions <tool>              # 显示可用版本 (改名from fetch)
vx use <tool>@<version>         # 切换到指定版本 (改名from switch)

# 搜索
vx search <query>               # 搜索可用工具
```

### 项目管理 (Project Management)

```bash
# 项目初始化和同步
vx init [--template=<name>]     # 初始化项目配置
vx sync [--check|--dry-run]     # 同步项目工具

# 工具执行
vx run <tool> [args...]         # 在项目环境中运行工具 (新增)
vx exec <command>               # 在项目环境中执行命令 (新增)
```

### 环境管理 (Environment Management)

```bash
# 虚拟环境管理 (重新组织venv命令)
vx env create <name> [--tools=<list>]  # 创建环境
vx env list [--current]                # 列出环境
vx env use <name>                       # 激活环境
vx env remove <name>                    # 删除环境
vx env current                          # 显示当前环境
```

### 配置和维护 (Configuration & Maintenance)

```bash
# 配置管理
vx config [get|set|list|edit]   # 配置管理
vx config get <key>             # 获取配置值
vx config set <key> <value>     # 设置配置值

# 系统维护
vx clean [--cache|--tools|--all]  # 清理系统 (改名from cleanup)
vx doctor                          # 诊断系统问题 (新增)
vx info [--system|--project]      # 显示系统信息 (改名from stats)
```

### Shell集成 (Shell Integration)

```bash
# Shell集成 (重新组织)
vx shell init [shell]           # 生成Shell初始化脚本
vx shell completions [shell]    # 生成Shell补全脚本
```

### 插件管理 (Plugin Management)

```bash
# 插件管理 (保持现有结构)
vx plugin list                  # 列出插件
vx plugin install <name>        # 安装插件
vx plugin remove <name>         # 删除插件
```

## 全局选项标准化

### 通用选项
```bash
-h, --help          # 显示帮助
-V, --version       # 显示版本
-v, --verbose       # 详细输出
-q, --quiet         # 静默模式
--dry-run           # 预览模式
--force             # 强制执行
--no-color          # 禁用颜色
--config <file>     # 指定配置文件
```

### 输出格式选项
```bash
--format <format>   # 输出格式: table, json, yaml
--output <file>     # 输出到文件
```

## 别名和简写

### 常用命令别名
```bash
vx i    -> vx install
vx rm   -> vx uninstall  
vx ls   -> vx list
vx up   -> vx update
vx cfg  -> vx config
```

## 向后兼容性

### 迁移策略
1. **阶段1**: 添加新命令，保留旧命令但显示弃用警告
2. **阶段2**: 默认使用新命令，旧命令显示迁移提示
3. **阶段3**: 移除旧命令（主版本更新）

### 弃用警告示例
```bash
$ vx remove node
Warning: 'vx remove' is deprecated. Use 'vx uninstall' instead.
This command will be removed in vx 1.0.0.
```

## 实施计划

### 优先级1 (高优先级)
- [ ] 重命名核心命令 (remove -> uninstall, where -> which, etc.)
- [ ] 统一命令命名约定
- [ ] 标准化全局选项

### 优先级2 (中优先级)  
- [ ] 重新组织环境管理命令
- [ ] 添加新的项目管理命令 (run, exec)
- [ ] 改进Shell集成命令

### 优先级3 (低优先级)
- [ ] 添加别名支持
- [ ] 实施弃用警告
- [ ] 添加诊断命令 (doctor)

## 参考资料

- [CLI Guidelines](https://clig.dev/)
- [12 Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46)
- [GNU Coding Standards](https://www.gnu.org/prep/standards/html_node/Command_002dLine-Interfaces.html)

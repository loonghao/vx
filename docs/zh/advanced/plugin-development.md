# 插件开发

vx 支持通过插件系统扩展功能。

## 插件概述

插件可以：

- 添加新的工具支持
- 自定义版本解析
- 扩展安装流程

## 插件结构

```
my-plugin/
├── plugin.toml
├── src/
│   └── lib.rs
└── Cargo.toml
```

## 配置文件

`plugin.toml`:

```toml
[plugin]
name = "my-tool"
version = "1.0.0"
description = "My custom tool support"

[tool]
name = "mytool"
aliases = ["mt"]
```

## 开发指南

详细的插件开发文档正在编写中。

## 参见

- [架构](/zh/advanced/architecture) - 了解 vx 架构
- [贡献指南](/zh/advanced/contributing) - 如何贡献

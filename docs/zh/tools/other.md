# 其他工具

vx 还支持许多其他开发工具。

## DevOps 工具

| 工具 | 描述 |
|------|------|
| `kubectl` | Kubernetes CLI |
| `helm` | Kubernetes 包管理器 |
| `terraform` | 基础设施即代码 |

## 实用工具

| 工具 | 描述 |
|------|------|
| `just` | 命令运行器 |
| `jq` | JSON 处理器 |
| `ripgrep` | 快速搜索工具 |

## 使用示例

```bash
# DevOps 工具
vx kubectl get pods
vx helm install my-release my-chart
vx terraform plan

# 实用工具
vx just build
vx jq '.name' package.json
vx rg "TODO" src/
```

## 项目配置

```toml
[tools]
just = "latest"
jq = "latest"

[scripts]
deploy = "kubectl apply -f k8s/"
```

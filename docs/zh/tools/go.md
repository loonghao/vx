# Go

vx 支持 Go 编程语言。

## 支持的工具

| 工具 | 描述 |
|------|------|
| `go` | Go 编程语言 |

## 使用示例

```bash
# 运行 Go
vx go version
vx go build
vx go test ./...
vx go run main.go

# 安装 Go 工具
vx go install golang.org/x/tools/gopls@latest
```

## 版本管理

```bash
# 安装特定版本
vx install go@1.21
vx install go@1.21.5
```

## 项目配置

```toml
[tools]
go = "1.21"

[scripts]
build = "go build -o app"
test = "go test ./..."
run = "go run main.go"
```

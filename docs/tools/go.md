# Go

vx provides full support for the Go programming language.

## Installation

```bash
vx install go 1.21
vx install `go@latest
```

## Version Specifiers

```bash
go 1.21          # Latest 1.21.x
go 1.21.5        # Exact version
go latest        # Latest stable
```

## Usage

### Basic Commands

```bash
vx go version
vx go env
vx go help
```

### Building

```bash
vx go build
vx go build -o myapp
vx go build -ldflags "-s -w" -o myapp
```

### Running

```bash
vx go run main.go
vx go run .
```

### Testing

```bash
vx go test ./...
vx go test -v ./...
vx go test -cover ./...
```

### Module Management

```bash
vx go mod init mymodule
vx go mod tidy
vx go mod download
vx go get github.com/gin-gonic/gin
```

### Installing Tools

```bash
vx go install golang.org/x/tools/gopls@latest
vx go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
```

## Project Configuration

```toml
[tools]
go = "1.21"

[scripts]
build = "go build -o app"
test = "go test ./..."
lint = "golangci-lint run"
run = "go run ."
```

## Common Workflows

### New Go Project

```bash
mkdir my-project
cd my-project
vx go mod init github.com/user/my-project
```

### Web Server with Gin

```bash
vx go mod init myserver
vx go get github.com/gin-gonic/gin
# Create main.go
vx go run .
```

### CLI Tool with Cobra

```bash
vx go mod init mycli
vx go get github.com/spf13/cobra
# Create main.go
vx go build -o mycli
```

## Cross-Compilation

```bash
# Linux
GOOS=linux GOARCH=amd64 vx go build -o app-linux

# Windows
GOOS=windows GOARCH=amd64 vx go build -o app.exe

# macOS
GOOS=darwin GOARCH=amd64 vx go build -o app-mac
```

## Environment Variables

Go-specific environment variables work as expected:

```bash
GOPROXY=direct vx go get github.com/user/repo
CGO_ENABLED=0 vx go build -o app
```

## Tips

1. **Use go mod tidy**: Keep dependencies clean
2. **Pin Go version**: Ensure team uses same version
3. **Use golangci-lint**: Comprehensive linting

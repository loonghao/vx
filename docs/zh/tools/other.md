# 其他工具

vx 还支持许多其他开发工具。

## 语言运行时

### Deno

安全的 JavaScript/TypeScript 运行时。

```bash
vx install deno latest

vx deno --version
vx deno run script.ts
vx deno compile script.ts
vx deno task dev
```

### Zig

Zig 编程语言。

```bash
vx install zig latest

vx zig version
vx zig build
vx zig run main.zig
```

### Java

Java 开发工具包。

```bash
vx install java 21
vx install java 17

vx java --version
vx javac Main.java
vx java Main
```

## 构建工具

### Vite

下一代前端工具。

```bash
vx install vite latest

vx vite
vx vite build
vx vite preview
```

### Just

命令运行器（类似 make，但更简单）。

```bash
vx install just latest

vx just --list
vx just build
vx just test
```

### Task (go-task)

任务运行器 / Make 的替代构建工具。

```bash
vx install task latest

vx task --version
vx task build
vx task test
vx task --list
```

### CMake

跨平台构建系统生成器。

```bash
vx install cmake latest

vx cmake --version
vx cmake -B build -S .
vx cmake --build build
vx cmake --install build
```

### Ninja

专注于速度的小型构建系统。

```bash
vx install ninja latest

vx ninja --version
vx ninja -C build
vx ninja -C build clean
```

### protoc

Protocol Buffers 编译器。

```bash
vx install protoc latest

vx protoc --version
vx protoc --cpp_out=. message.proto
vx protoc --python_out=. message.proto
vx protoc --go_out=. message.proto
```

## DevOps 工具

### Terraform

基础设施即代码。

```bash
vx install terraform latest

vx terraform --version
vx terraform init
vx terraform plan
vx terraform apply
```

### kubectl

Kubernetes 命令行工具。

```bash
vx install kubectl latest

vx kubectl version
vx kubectl get pods
vx kubectl apply -f deployment.yaml
```

### Helm

Kubernetes 包管理器。

```bash
vx install helm latest

vx helm version
vx helm install my-release chart/
vx helm upgrade my-release chart/
```

### Docker

容器运行时和工具。

```bash
vx install docker latest

vx docker --version
vx docker build -t myapp .
vx docker run -it myapp
vx docker compose up -d
```

## 云 CLI 工具

### AWS CLI

亚马逊云服务命令行界面。

```bash
vx install awscli latest

vx aws --version
vx aws configure
vx aws s3 ls
vx aws ec2 describe-instances
```

### Azure CLI

微软 Azure 命令行界面。

```bash
vx install azcli latest

vx az --version
vx az login
vx az group list
vx az vm list
```

### gcloud

谷歌云平台命令行界面。

```bash
vx install gcloud latest

vx gcloud --version
vx gcloud auth login
vx gcloud projects list
vx gcloud compute instances list
```

## 代码质量工具

### pre-commit

管理预提交钩子的框架。

```bash
vx install pre-commit latest

vx pre-commit --version
vx pre-commit install
vx pre-commit run --all-files
vx pre-commit autoupdate
```

## 编辑器 & IDE

### VS Code

Visual Studio Code（命令行）。

```bash
vx install vscode latest

vx code .
vx code --install-extension ms-python.python
```

## 专业工具

### rez

VFX/动画行业的包管理系统。

```bash
vx install rez latest

vx rez --version
vx rez env package
```

### rcedit

Windows 资源编辑器。

```bash
vx install rcedit latest

vx rcedit app.exe --set-icon icon.ico
vx rcedit app.exe --set-version-string "ProductName" "My App"
```

## 项目配置示例

```toml
[tools]
deno = "latest"
terraform = "1.6"
kubectl = "latest"
helm = "latest"
docker = "latest"
awscli = "latest"
pre-commit = "latest"
cmake = "latest"
task = "latest"

[scripts]
dev = "deno task dev"
deploy = "terraform apply -auto-approve"
k8s-status = "kubectl get pods -A"
docker-build = "docker build -t myapp ."
lint = "pre-commit run --all-files"
build = "cmake -B build && cmake --build build"
```

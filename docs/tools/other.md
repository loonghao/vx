# Other Tools

vx supports various other development tools.

## Language Runtimes

### Deno

Secure JavaScript/TypeScript runtime.

```bash
vx install deno latest

vx deno --version
vx deno run script.ts
vx deno compile script.ts
vx deno task dev
```

### Zig

Zig programming language.

```bash
vx install zig latest

vx zig version
vx zig build
vx zig run main.zig
```

### Java

Java Development Kit.

```bash
vx install java 21
vx install java 17

vx java --version
vx javac Main.java
vx java Main
```

### .NET SDK

.NET SDK for C#, F#, and VB.NET development.

```bash
vx install dotnet latest

vx dotnet --version
vx dotnet new console -n MyApp
vx dotnet build
vx dotnet run
vx dotnet test
vx dotnet publish -c Release
```

**Key Features:**
- Cross-platform development (Windows, macOS, Linux)
- Support for C#, F#, and Visual Basic
- Built-in package management (NuGet)
- Web, desktop, mobile, cloud, and IoT applications

## Build Tools

### Vite

Next generation frontend tooling.

```bash
vx install vite latest

vx vite
vx vite build
vx vite preview
```

### Just

Command runner (like make, but simpler).

```bash
vx install just latest

vx just --list
vx just build
vx just test
```

### Task (go-task)

Task runner / build tool alternative to Make.

```bash
vx install task latest

vx task --version
vx task build
vx task test
vx task --list
```

### CMake

Cross-platform build system generator.

```bash
vx install cmake latest

vx cmake --version
vx cmake -B build -S .
vx cmake --build build
vx cmake --install build
```

### Ninja

Small build system with a focus on speed.

```bash
vx install ninja latest

vx ninja --version
vx ninja -C build
vx ninja -C build clean
```

### protoc

Protocol Buffers compiler.

```bash
vx install protoc latest

vx protoc --version
vx protoc --cpp_out=. message.proto
vx protoc --python_out=. message.proto
vx protoc --go_out=. message.proto
```

## DevOps Tools

### Terraform

Infrastructure as Code.

```bash
vx install terraform latest

vx terraform --version
vx terraform init
vx terraform plan
vx terraform apply
```

### kubectl

Kubernetes CLI.

```bash
vx install kubectl latest

vx kubectl version
vx kubectl get pods
vx kubectl apply -f deployment.yaml
```

### Helm

Kubernetes package manager.

```bash
vx install helm latest

vx helm version
vx helm install my-release chart/
vx helm upgrade my-release chart/
```

### Docker

Container runtime and tooling.

```bash
vx install docker latest

vx docker --version
vx docker build -t myapp .
vx docker run -it myapp
vx docker compose up -d
```

## Cloud CLI Tools

### AWS CLI

Amazon Web Services command-line interface.

```bash
vx install awscli latest

vx aws --version
vx aws configure
vx aws s3 ls
vx aws ec2 describe-instances
```

### Azure CLI

Microsoft Azure command-line interface.

```bash
vx install azcli latest

vx az --version
vx az login
vx az group list
vx az vm list
```

### gcloud

Google Cloud Platform command-line interface.

```bash
vx install gcloud latest

vx gcloud --version
vx gcloud auth login
vx gcloud projects list
vx gcloud compute instances list
```

## Code Quality Tools

### pre-commit

Framework for managing pre-commit hooks.

```bash
vx install pre-commit latest

vx pre-commit --version
vx pre-commit install
vx pre-commit run --all-files
vx pre-commit autoupdate
```

## Editor & IDE

### VS Code

Visual Studio Code (CLI).

```bash
vx install vscode latest

vx code .
vx code --install-extension ms-python.python
```

## Specialized Tools

### rez

Package management system for VFX/animation.

```bash
vx install rez latest

vx rez --version
vx rez env package
```

### rcedit

Windows resource editor.

```bash
vx install rcedit latest

vx rcedit app.exe --set-icon icon.ico
vx rcedit app.exe --set-version-string "ProductName" "My App"
```

## Project Configuration Example

```toml
[tools]
deno = "latest"
dotnet = "latest"
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
dotnet-build = "dotnet build"
dotnet-test = "dotnet test"
```

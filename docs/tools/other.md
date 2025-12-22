# Other Tools

vx supports various other development tools.

## Deno

Secure JavaScript/TypeScript runtime.

```bash
vx install deno latest

vx deno --version
vx deno run script.ts
vx deno compile script.ts
vx deno task dev
```

## Zig

Zig programming language.

```bash
vx install zig latest

vx zig version
vx zig build
vx zig run main.zig
```

## Java

Java Development Kit.

```bash
vx install java 21
vx install java 17

vx java --version
vx javac Main.java
vx java Main
```

## Vite

Next generation frontend tooling.

```bash
vx install vite latest

vx vite
vx vite build
vx vite preview
```

## Just

Command runner (like make, but simpler).

```bash
vx install just latest

vx just --list
vx just build
vx just test
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

## VS Code

Visual Studio Code (CLI).

```bash
vx install vscode latest

vx code .
vx code --install-extension ms-python.python
```

## rez

Package management system for VFX/animation.

```bash
vx install rez latest

vx rez --version
vx rez env package
```

## rcedit

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
terraform = "1.6"
kubectl = "latest"
helm = "latest"

[scripts]
dev = "deno task dev"
deploy = "terraform apply -auto-approve"
k8s-status = "kubectl get pods -A"
```

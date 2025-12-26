# DevOps 工具

vx 支持各种 DevOps 和基础设施工具。

## 基础设施即代码

### Terraform

HashiCorp Terraform 基础设施即代码工具。

```bash
vx install terraform latest

vx terraform --version
vx terraform init
vx terraform plan
vx terraform apply
vx terraform destroy
```

**主要特性：**

- 多云基础设施管理
- 状态管理
- 模块支持

## 容器工具

### Docker

Docker CLI 容器管理工具。

```bash
vx install docker latest

vx docker --version
vx docker build -t myapp .
vx docker run -it myapp
vx docker compose up -d
vx docker ps
```

**注意：** 这只安装 Docker CLI。您仍需要在系统上运行 Docker Engine 或 Docker Desktop。

## Kubernetes

### kubectl

Kubernetes 命令行工具。

```bash
vx install kubectl latest

vx kubectl version
vx kubectl get pods
vx kubectl get services
vx kubectl apply -f deployment.yaml
vx kubectl logs pod-name
vx kubectl exec -it pod-name -- /bin/sh
```

### Helm

Kubernetes 包管理器。

```bash
vx install helm latest

vx helm version
vx helm repo add stable https://charts.helm.sh/stable
vx helm search repo nginx
vx helm install my-release chart/
vx helm upgrade my-release chart/
vx helm list
```

## 版本控制

### Git

分布式版本控制系统。

```bash
vx install git latest

vx git --version
vx git clone https://github.com/user/repo.git
vx git status
vx git add .
vx git commit -m "message"
vx git push
```

**平台支持：**

- Windows：MinGit（便携版 Git）
- Linux/macOS：建议使用系统包管理器

## 项目配置示例

```toml
[tools]
terraform = "1.6"
docker = "latest"
kubectl = "latest"
helm = "latest"
git = "latest"

[scripts]
deploy = "terraform apply -auto-approve"
k8s-status = "kubectl get pods -A"
docker-build = "docker build -t myapp ."
helm-deploy = "helm upgrade --install myapp ./chart"
```

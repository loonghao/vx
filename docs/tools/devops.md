# DevOps Tools

vx supports various DevOps and infrastructure tools.

## Infrastructure as Code

### Terraform

HashiCorp Terraform for infrastructure as code.

```bash
vx install terraform latest

vx terraform --version
vx terraform init
vx terraform plan
vx terraform apply
vx terraform destroy
```

**Key Features:**

- Multi-cloud infrastructure management
- State management
- Module support

## Container Tools

### Docker

Docker CLI for container management.

```bash
vx install docker latest

vx docker --version
vx docker build -t myapp .
vx docker run -it myapp
vx docker compose up -d
vx docker ps
```

**Note:** This installs the Docker CLI. You still need Docker Engine or Docker Desktop running on your system.

## Kubernetes

### kubectl

Kubernetes command-line tool.

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

The package manager for Kubernetes.

```bash
vx install helm latest

vx helm version
vx helm repo add stable https://charts.helm.sh/stable
vx helm search repo nginx
vx helm install my-release chart/
vx helm upgrade my-release chart/
vx helm list
```

## Version Control

### Git

Distributed version control system.

```bash
vx install git latest

vx git --version
vx git clone https://github.com/user/repo.git
vx git status
vx git add .
vx git commit -m "message"
vx git push
```

**Platform Support:**

- Windows: MinGit (portable Git)
- Linux/macOS: System package manager recommended

## Project Configuration Example

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

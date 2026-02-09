# DevOps 工具

vx 支持各种 DevOps 和基础设施工具。

## 基础设施即代码

### Terraform

HashiCorp Terraform 基础设施即代码工具。

```bash
vx install `terraform@latest

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
vx install `docker@latest

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
vx install `kubectl@latest

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
vx install `helm@latest

vx helm version
vx helm repo add stable https://charts.helm.sh/stable
vx helm search repo nginx
vx helm install my-release chart/
vx helm upgrade my-release chart/
vx helm list
```

## 工作流引擎

### Dagu

基于 DAG 的工作流执行器，内置 Web UI。适用于编排多步骤管道、定时任务和跨工具工作流。

```bash
vx install dagu@latest

# 启动 Web UI 仪表板 (http://localhost:8080)
vx dagu server

# 运行工作流
vx dagu start my-workflow
vx dagu status my-workflow
vx dagu stop my-workflow

# 试运行（验证但不执行）
vx dagu dry my-workflow
```

**主要特性：**

- 基于 YAML 的 DAG 工作流定义
- 内置 Web UI 用于监控和管理
- 步骤依赖与并行执行
- Cron 定时调度支持
- 重试、超时和条件执行
- 环境变量和参数传递

**工作流示例（build-pipeline.yaml）：**

```yaml
# build-pipeline.yaml
params:
  - ENV: production

steps:
  - name: lint
    command: uvx ruff check .

  - name: test
    command: uv run pytest
    depends:
      - lint

  - name: build-backend
    command: cargo build --release
    depends:
      - test

  - name: build-frontend
    command: npm run build
    depends:
      - test

  - name: deploy
    command: kubectl apply -f k8s/
    depends:
      - build-backend
      - build-frontend
    preconditions:
      - condition: "`echo $ENV`"
        expected: "production"
```

运行：

```bash
vx dagu start build-pipeline
```

**在 Dagu 工作流中使用 vx 管理的工具：**

由于 vx 的子进程 PATH 继承，所有 vx 管理的工具在 Dagu 步骤中都可以直接使用，无需 `vx` 前缀：

```yaml
# 所有工具通过 vx 的 PATH 继承可用
steps:
  - name: python-analysis
    command: uv run python analyze.py

  - name: go-build
    command: go build -o server ./cmd/server
    depends:
      - python-analysis

  - name: node-report
    command: npx generate-report
    depends:
      - go-build
```

只需通过 vx 启动 Dagu：

```bash
vx dagu server   # 工作流步骤中可使用所有 vx 工具
```

**定时工作流：**

```yaml
# scheduled-backup.yaml
schedule: "0 2 * * *"   # 每天凌晨 2 点运行

steps:
  - name: backup-db
    command: pg_dump $DATABASE_URL > backup-$(date +%Y%m%d).sql

  - name: upload
    command: aws s3 cp backup-*.sql s3://backups/
    depends:
      - backup-db

  - name: cleanup
    command: find . -name "backup-*.sql" -mtime +7 -delete
    depends:
      - upload
```

**vx.toml 集成：**

```toml
[tools]
dagu = "latest"

[scripts]
# 启动 Dagu 仪表板
dashboard = "dagu server"

# 运行特定工作流
pipeline = "dagu start build-pipeline"
deploy = "dagu start deploy-workflow"
```

## 版本控制

### Git

分布式版本控制系统。

```bash
vx install `git@latest

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
dagu = "latest"

[scripts]
deploy = "terraform apply -auto-approve"
k8s-status = "kubectl get pods -A"
docker-build = "docker build -t myapp ."
helm-deploy = "helm upgrade --install myapp ./chart"
dashboard = "dagu server"
pipeline = "dagu start build-pipeline"
```

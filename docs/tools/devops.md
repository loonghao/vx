# DevOps Tools

vx supports various DevOps and infrastructure tools.

## Infrastructure as Code

### Terraform

HashiCorp Terraform for infrastructure as code.

```bash
vx install `terraform@latest

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
vx install `docker@latest

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
vx install `kubectl@latest

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
vx install `helm@latest

vx helm version
vx helm repo add stable https://charts.helm.sh/stable
vx helm search repo nginx
vx helm install my-release chart/
vx helm upgrade my-release chart/
vx helm list
```

## Workflow Engines

### Dagu

DAG-based workflow executor with a built-in web UI. Perfect for orchestrating multi-step pipelines, scheduled jobs, and cross-tool workflows.

```bash
vx install dagu@latest

# Start the web UI dashboard (http://localhost:8080)
vx dagu server

# Run a workflow
vx dagu start my-workflow
vx dagu status my-workflow
vx dagu stop my-workflow

# Dry run (validate without executing)
vx dagu dry my-workflow
```

**Key Features:**

- YAML-based DAG workflow definitions
- Built-in web UI for monitoring and management
- Step dependencies with parallel execution
- Cron scheduling support
- Retry, timeout, and conditional execution
- Environment variable and parameter passing

**Example Workflow (build-pipeline.yaml):**

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

Run with:

```bash
vx dagu start build-pipeline
```

**Using vx-managed tools in Dagu workflows:**

Because of vx's subprocess PATH inheritance, all vx-managed tools are available inside Dagu steps without the `vx` prefix:

```yaml
# All these tools are available via vx's PATH
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

Just start Dagu through vx:

```bash
vx dagu server   # All vx tools available in workflow steps
```

**Scheduled Workflows:**

```yaml
# scheduled-backup.yaml
schedule: "0 2 * * *"   # Run at 2 AM daily

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

**vx.toml Integration:**

```toml
[tools]
dagu = "latest"

[scripts]
# Start Dagu dashboard
dashboard = "dagu server"

# Run specific workflows
pipeline = "dagu start build-pipeline"
deploy = "dagu start deploy-workflow"
```

## Version Control

### Git

Distributed version control system.

```bash
vx install `git@latest

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
dagu = "latest"

[scripts]
deploy = "terraform apply -auto-approve"
k8s-status = "kubectl get pods -A"
docker-build = "docker build -t myapp ."
helm-deploy = "helm upgrade --install myapp ./chart"
dashboard = "dagu server"
pipeline = "dagu start build-pipeline"
```

# 云 CLI 工具

vx 支持主要云服务商的命令行工具。

## AWS CLI

Amazon Web Services 命令行界面（v2）。

```bash
vx install `aws@latest

vx aws --version
vx aws configure
vx aws s3 ls
vx aws ec2 describe-instances
vx aws lambda list-functions
vx aws sts get-caller-identity
```

**主要特性：**

- 完整的 AWS 服务覆盖
- 配置文件管理
- SSO 支持

## Azure CLI

Microsoft Azure 命令行界面。

```bash
vx install `az@latest

vx az --version
vx az login
vx az account list
vx az group list
vx az vm list
vx az storage account list
```

**主要特性：**

- Azure 资源管理
- 交互式登录
- 服务主体支持

## Google Cloud CLI

Google Cloud Platform 命令行界面。

```bash
vx install `gcloud@latest

vx gcloud --version
vx gcloud auth login
vx gcloud config set project PROJECT_ID
vx gcloud projects list
vx gcloud compute instances list
vx gcloud container clusters list
```

**主要特性：**

- GCP 资源管理
- 多项目支持
- Cloud SDK 组件

## 多云配置

```toml
[tools]
aws = "latest"
az = "latest"
gcloud = "latest"
terraform = "latest"

[scripts]
aws-login = "aws sso login"
az-login = "az login"
gcloud-login = "gcloud auth login"
```

## 最佳实践

1. **凭证管理**：使用环境变量或云特定的凭证文件
2. **配置文件切换**：为不同账户/环境配置多个配置文件
3. **版本锁定**：在生产环境中锁定 CLI 版本以保持一致性

```toml
[tools]
# 生产环境锁定特定版本
aws = "2.15"
az = "2.55"
gcloud = "460"
```

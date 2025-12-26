# Cloud CLI Tools

vx supports major cloud provider command-line interfaces.

## AWS CLI

Amazon Web Services command-line interface (v2).

```bash
vx install aws latest

vx aws --version
vx aws configure
vx aws s3 ls
vx aws ec2 describe-instances
vx aws lambda list-functions
vx aws sts get-caller-identity
```

**Key Features:**

- Full AWS service coverage
- Profile management
- SSO support

## Azure CLI

Microsoft Azure command-line interface.

```bash
vx install az latest

vx az --version
vx az login
vx az account list
vx az group list
vx az vm list
vx az storage account list
```

**Key Features:**

- Azure resource management
- Interactive login
- Service principal support

## Google Cloud CLI

Google Cloud Platform command-line interface.

```bash
vx install gcloud latest

vx gcloud --version
vx gcloud auth login
vx gcloud config set project PROJECT_ID
vx gcloud projects list
vx gcloud compute instances list
vx gcloud container clusters list
```

**Key Features:**

- GCP resource management
- Multiple project support
- Cloud SDK components

## Multi-Cloud Configuration

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

## Best Practices

1. **Credential Management**: Use environment variables or cloud-specific credential files
2. **Profile Switching**: Configure multiple profiles for different accounts/environments
3. **Version Pinning**: Pin CLI versions in production for consistency

```toml
[tools]
# Pin specific versions for production
aws = "2.15"
az = "2.55"
gcloud = "460"
```

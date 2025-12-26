# Deployment Guide for Static Web Lambda

This guide covers all deployment build options for the Static Web Lambda project, including Docker-based builds, local cross-compilation, and AWS CodeBuild integration.

## Overview

The Static Web Lambda project supports multiple build methods to create AWS Lambda-compatible deployment packages:

1. **Docker Build** (Recommended) - Cross-platform compatible, uses containerized Linux environment
2. **Local Cross-compilation** - Uses local Rust toolchain with Linux target
3. **Native Build** - For local testing only (not Lambda-compatible unless on Amazon Linux)
4. **AWS CodeBuild** - Cloud-based build service with native Linux environment

## Quick Start

The fastest way to create a deployment package:

```bash
# Build using Docker (recommended)
./scripts/build-lambda.sh docker

# Or build using local cross-compilation
./scripts/build-lambda.sh local
```

This will create a `lambda-deployment.zip` file ready for AWS Lambda deployment.

## Build Methods

### 1. Docker Build (Recommended)

Docker builds provide the most reliable cross-platform compatibility by using a Linux container environment.

**Prerequisites:**
- Docker installed and running
- Project source code

**Usage:**
```bash
# Basic Docker build
./scripts/build-lambda.sh docker

# Clean build with verbose output
./scripts/build-lambda.sh docker --clean --verbose

# Use custom Docker image
DOCKER_IMAGE=rust:1.70-slim ./scripts/build-lambda.sh docker
```

**Advantages:**
- Works on any platform (macOS, Windows, Linux)
- Consistent build environment
- No local cross-compilation setup required
- Reproducible builds

**How it works:**
1. Uses official Rust Docker image (`rust:1.83`) with explicit `linux/amd64` platform
2. Installs Linux target (`x86_64-unknown-linux-gnu`)
3. Compiles project in containerized Linux environment
4. Creates deployment package with `bootstrap` executable

**Platform Compatibility:**
- Works on Apple Silicon (M1/M2) Macs by using `--platform linux/amd64`
- Works on Intel Macs and Linux systems
- Works on Windows with Docker Desktop

### 2. Local Cross-compilation

Uses your local Rust installation with Linux target for faster builds.

**Prerequisites:**
- Rust toolchain installed locally
- Linux target installed (`x86_64-unknown-linux-gnu`)

**Setup:**
```bash
# Install Linux target (done automatically by build script)
rustup target add x86_64-unknown-linux-gnu
```

**Usage:**
```bash
# Basic local build
./scripts/build-lambda.sh local

# Clean build
./scripts/build-lambda.sh local --clean

# Custom target architecture
TARGET_ARCH=x86_64-unknown-linux-musl ./scripts/build-lambda.sh local
```

**Advantages:**
- Faster builds (no Docker overhead)
- Uses local Rust cache
- Direct access to build artifacts

**Limitations:**
- May require additional setup on some platforms (especially macOS cross-compiling to Linux)
- Platform-specific linking issues possible
- Requires local Rust toolchain with cross-compilation support

**macOS Note:** Local cross-compilation from macOS to Linux may require additional linker setup. Docker build is recommended for macOS users.

### 3. Native Build (Testing Only)

Builds using your native platform. **Only use for local testing - will not work on AWS Lambda unless built on Amazon Linux.**

**Usage:**
```bash
# Native build (testing only)
./scripts/build-lambda.sh native
```

**Warning:** Native builds are only compatible with AWS Lambda if built on Amazon Linux. Use Docker or local cross-compilation for actual deployments.

### 4. AWS CodeBuild

Generates configuration files for AWS CodeBuild service to handle builds in the cloud.

**Usage:**
```bash
# Generate CodeBuild configuration
./scripts/build-lambda.sh codebuild
```

This creates:
- `buildspec.yml` - CodeBuild build specification
- `codebuild-project-template.json` - CodeBuild project template

**Setup AWS CodeBuild:**

1. **Create S3 bucket for artifacts:**
   ```bash
   aws s3 mb s3://your-lambda-build-artifacts
   ```

2. **Create CodeBuild service role:**
   ```bash
   aws iam create-role --role-name codebuild-static-web-lambda-service-role \
     --assume-role-policy-document file://codebuild-trust-policy.json
   ```

3. **Create CodeBuild project:**
   ```bash
   # Update codebuild-project-template.json with your details
   aws codebuild create-project --cli-input-json file://codebuild-project-template.json
   ```

4. **Start build:**
   ```bash
   aws codebuild start-build --project-name static-web-lambda-build
   ```

## Build Script Options

The `scripts/build-lambda.sh` script supports various options:

### Command Line Options

```bash
./scripts/build-lambda.sh [OPTIONS] [BUILD_METHOD]
```

**Build Methods:**
- `docker` - Build using Docker (recommended)
- `local` - Build using local cross-compilation
- `native` - Build using native compilation (testing only)
- `codebuild` - Generate AWS CodeBuild configuration

**Options:**
- `-h, --help` - Show help message
- `-v, --verbose` - Enable verbose output
- `-c, --clean` - Clean build artifacts before building
- `--validate` - Validate build environment only
- `--package-only` - Create deployment package from existing binary

### Environment Variables

Customize build behavior with environment variables:

```bash
# Custom Docker image
DOCKER_IMAGE=rust:1.70-slim ./scripts/build-lambda.sh docker

# Custom target architecture
TARGET_ARCH=x86_64-unknown-linux-musl ./scripts/build-lambda.sh local

# Additional cargo flags
CARGO_FLAGS="--features production" ./scripts/build-lambda.sh docker
```

### Examples

```bash
# Validate build environment
./scripts/build-lambda.sh --validate

# Clean Docker build with verbose output
./scripts/build-lambda.sh docker --clean --verbose

# Local build with custom target
TARGET_ARCH=x86_64-unknown-linux-musl ./scripts/build-lambda.sh local

# Create package from existing binary
./scripts/build-lambda.sh --package-only

# Generate CodeBuild configuration
./scripts/build-lambda.sh codebuild
```

## Build Validation and Error Handling

The build script includes comprehensive validation and error handling:

### Environment Validation

Before building, the script validates:
- Cargo.toml exists (correct directory)
- Rust/Cargo installed
- Docker available (for Docker builds)
- Required source files exist

### Build Validation

During and after building:
- Binary creation verification
- File permissions validation
- Package size reporting
- ZIP contents verification

### Error Handling

The script handles common errors:
- Missing dependencies
- Docker daemon not running
- Cross-compilation target not installed
- Build failures with clear error messages
- Package creation failures

## Deployment Package Structure

The build process creates a deployment package with this structure:

```
lambda-deployment.zip
└── bootstrap          # Executable binary (required name for Lambda)
```

**Key Requirements:**
- Binary must be named `bootstrap` (AWS Lambda requirement)
- Binary must have executable permissions (`chmod +x`)
- Binary must be compiled for Linux (`x86_64-unknown-linux-gnu`)
- Package must be in ZIP format

## Integration with Terraform

The deployment package integrates with Terraform for infrastructure deployment:

```hcl
# terraform/main.tf
resource "aws_lambda_function" "static_web" {
  filename         = "../lambda-deployment.zip"
  function_name    = "static-web-lambda"
  role            = aws_iam_role.lambda_role.arn
  handler         = "bootstrap"
  runtime         = "provided.al2"
  
  # Ensure package is rebuilt when source changes
  source_code_hash = filebase64sha256("../lambda-deployment.zip")
}
```

## Continuous Integration

### GitHub Actions Example

```yaml
# .github/workflows/deploy.yml
name: Build and Deploy Lambda

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Lambda package
        run: ./scripts/build-lambda.sh docker
        
      - name: Deploy with Terraform
        run: |
          cd terraform
          terraform init
          terraform plan
          terraform apply -auto-approve
```

### AWS CodePipeline Integration

1. **Source Stage:** GitHub repository
2. **Build Stage:** CodeBuild project (using generated `buildspec.yml`)
3. **Deploy Stage:** CloudFormation or Terraform deployment

## Troubleshooting

### Common Issues

**Docker build fails:**
```bash
# Check Docker is running
docker info

# Try with verbose output
./scripts/build-lambda.sh docker --verbose
```

**Cross-compilation fails:**
```bash
# Install Linux target
rustup target add x86_64-unknown-linux-gnu

# Clean and retry
./scripts/build-lambda.sh local --clean
```

**Package too large:**
```bash
# Check binary size
ls -lh target/x86_64-unknown-linux-gnu/release/static-web-lambda

# Optimize build
CARGO_FLAGS="--release" ./scripts/build-lambda.sh docker
```

**Lambda deployment fails:**
- Ensure binary is named `bootstrap`
- Verify executable permissions
- Check ZIP file structure
- Confirm Linux target compilation

### Debug Information

Enable verbose output for debugging:
```bash
./scripts/build-lambda.sh docker --verbose
```

This shows:
- Detailed build commands
- Docker container execution
- File operations
- Package creation steps

## Performance Optimization

### Build Speed

- Use local builds for faster iteration
- Enable Docker BuildKit for faster Docker builds
- Use build caches in CI/CD pipelines

### Package Size

- Use release builds (`--release`)
- Strip debug symbols
- Consider `musl` target for smaller binaries
- Optimize dependencies in `Cargo.toml`

### Lambda Performance

- Use `provided.al2` runtime for better cold start performance
- Optimize binary size for faster deployment
- Consider provisioned concurrency for consistent performance

## Security Considerations

### Build Security

- Use official Rust Docker images
- Pin Docker image versions for reproducible builds
- Validate build artifacts before deployment
- Use secure CI/CD pipelines

### Deployment Security

- Store deployment packages in secure S3 buckets
- Use IAM roles with least privilege
- Enable CloudTrail for deployment auditing
- Scan deployment packages for vulnerabilities

## Next Steps

After creating your deployment package:

1. **Test locally:** Use the native build for local testing
2. **Deploy infrastructure:** Use Terraform to create AWS resources
3. **Deploy function:** Upload the deployment package to Lambda
4. **Test deployment:** Verify the function works in AWS
5. **Set up monitoring:** Configure CloudWatch logs and metrics

For infrastructure deployment, see the Terraform configuration in the `terraform/` directory.
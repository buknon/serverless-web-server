# AWS Lambda Build Setup

This document describes the current build strategy for creating AWS Lambda-compatible deployment packages.

## Current Build Strategy

The project uses **Amazon Linux 2 Docker builds** to ensure complete compatibility with AWS Lambda runtime environment. This approach eliminates glibc version mismatches and provides consistent, reliable builds across all platforms.

### Docker-based Build (Recommended)

The project includes a custom `Dockerfile.build` that uses Amazon Linux 2 as the base image - the same operating system used by AWS Lambda runtime.

```bash
# Build Lambda deployment package using Docker
./scripts/build-lambda.sh docker

# This creates lambda-deployment.zip ready for deployment
```

**Key Benefits:**
- **Complete Compatibility**: Uses same OS as AWS Lambda (Amazon Linux 2)
- **No glibc Issues**: Native compilation eliminates version mismatches
- **Cross-Platform**: Works on macOS (Intel/Apple Silicon), Windows, and Linux
- **Consistent Builds**: Same environment every time
- **No Setup Required**: No cross-compilation toolchain installation needed

### How It Works

1. **Base Image**: Uses `amazonlinux:2` - identical to AWS Lambda runtime
2. **Rust Installation**: Installs Rust 1.83 natively on Amazon Linux 2
3. **Native Compilation**: Compiles directly on target OS (no cross-compilation)
4. **Package Creation**: Creates `lambda-deployment.zip` with `bootstrap` binary

### Alternative Build Methods

#### Local Cross-compilation
For faster iteration during development:

```bash
# Build using local Rust toolchain with Linux target
./scripts/build-lambda.sh local
```

**Requirements:**
- Rust toolchain with `x86_64-unknown-linux-gnu` target
- May require additional linker setup on some platforms

#### AWS CodeBuild
For CI/CD pipelines:

```bash
# Generate CodeBuild configuration
./scripts/build-lambda.sh codebuild
```

This creates `buildspec.yml` for AWS CodeBuild integration.

## Migration from Cross-compilation

This project previously used cross-compilation but migrated to the Docker approach for better reliability:

### What Changed
- **Before**: Cross-compilation from host OS to Linux target
- **After**: Native compilation on Amazon Linux 2 using Docker
- **Result**: Eliminated glibc compatibility issues

### Benefits of Migration
- ✅ No more glibc version mismatches
- ✅ Consistent builds across all development platforms
- ✅ Simplified setup (no cross-compilation toolchain required)
- ✅ Faster builds (no cross-compilation overhead)
- ✅ Better debugging (native compilation provides clearer error messages)

## Local Development

For local development and testing, use native compilation:

```bash
# Build and test locally
cargo build
cargo test

# Run local development server
cargo run -- --mode local
```

## Deployment Workflow

1. **Develop locally** using native compilation
2. **Build for Lambda** using Docker approach
3. **Deploy** the generated `lambda-deployment.zip`

```bash
# Complete workflow
cargo test                           # Test locally
./scripts/build-lambda.sh docker     # Build for Lambda
# Deploy lambda-deployment.zip to AWS
```

## Troubleshooting

### Docker Issues
```bash
# Ensure Docker is running
docker info

# Build with verbose output
./scripts/build-lambda.sh docker --verbose
```

### Build Failures
```bash
# Clean build
./scripts/build-lambda.sh docker --clean

# Validate environment
./scripts/build-lambda.sh --validate
```

## File Structure

The build process creates:
```
lambda-deployment.zip
└── bootstrap          # Executable binary (required name for Lambda)
```

This package is ready for deployment to AWS Lambda using any deployment method (AWS CLI, CDK, Terraform, Console).
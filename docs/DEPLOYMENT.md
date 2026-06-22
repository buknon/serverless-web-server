# Deployment Guide for Static Web Lambda

This guide covers building, testing locally, and deploying the Static Web Lambda project using cargo-lambda and Terraform.

## Overview

The project uses [cargo-lambda](https://www.cargo-lambda.info/) to build, test, and package the Lambda function. cargo-lambda handles cross-compilation to ARM64 Linux internally (using Zig as a linker), produces a ready-to-deploy ZIP artifact, and provides a local Lambda emulator for development. No Docker, shell scripts, or manual packaging steps are required.

**Build pipeline:**

```
cargo lambda build → bootstrap.zip → terraform apply → AWS Lambda (ARM64, provided.al2023)
```

## Prerequisites

Install cargo-lambda using one of:

```bash
# Via pip (works everywhere, recommended for CI)
pip3 install cargo-lambda

# Via Homebrew (macOS/Linux)
brew install cargo-lambda
```

You also need:
- Rust toolchain (stable) — install via [rustup](https://rustup.rs/)
- Terraform — for infrastructure deployment
- AWS credentials configured — for deploying to AWS

## Build Command

Build the Lambda deployment artifact:

```bash
cargo lambda build --release --arm64 --output-format zip
```

Or use the Makefile shortcut:

```bash
make build-lambda
```

This produces:

```
target/lambda/static-web-lambda/bootstrap.zip
```

cargo-lambda automatically:
- Cross-compiles to `aarch64-unknown-linux-gnu` (ARM64 Linux)
- Names the binary `bootstrap` (required by Lambda custom runtimes)
- Strips debug symbols for optimal size
- Packages the binary into a ZIP file

## Build Artifact Structure

The output is a single ZIP file containing the compiled binary:

```
target/lambda/static-web-lambda/bootstrap.zip
└── bootstrap          # ARM64 Linux ELF executable
```

| Property | Value |
|----------|-------|
| Path | `target/lambda/static-web-lambda/bootstrap.zip` |
| Binary name | `bootstrap` |
| Architecture | ARM64 (`aarch64-unknown-linux-gnu`) |
| Format | ZIP containing a single executable |

## Local Testing

cargo-lambda provides a local Lambda emulator for development. This is the recommended way to test changes before deploying.

### Start the Local Emulator

```bash
cargo lambda watch
```

Or via Makefile:

```bash
make watch
```

This starts a local Lambda emulator at `http://localhost:9000` with:
- **Hot-reload** — automatically recompiles when source files change
- **Lambda-compatible event handling** — accepts the same event payloads as AWS Lambda
- **No Docker required** — runs natively on your machine

### Invoke the Local Function

In a separate terminal, send a test event:

```bash
cargo lambda invoke static-web-lambda --data-ascii '{
  "httpMethod": "GET",
  "path": "/",
  "requestContext": {"http": {"method": "GET", "path": "/"}}
}'
```

Or via Makefile:

```bash
make invoke
```

The emulator returns the same response structure your function produces in AWS.

## Terraform Deployment

The Terraform configuration in `terraform/` deploys the Lambda function to AWS. It references the cargo-lambda build artifact directly.

### Deploy

```bash
# Build and deploy in one step
make build-deploy

# Or separately:
make build-lambda
make deploy
```

The `make deploy` target runs:

```bash
cd terraform && terraform apply
```

### Terraform Integration

The `terraform/lambda.tf` configuration references the cargo-lambda output:

```hcl
resource "aws_lambda_function" "static_web_lambda" {
  filename         = "${path.module}/../target/lambda/static-web-lambda/bootstrap.zip"
  source_code_hash = filebase64sha256("${path.module}/../target/lambda/static-web-lambda/bootstrap.zip")

  architectures = ["arm64"]
  runtime       = "provided.al2023"
  handler       = "bootstrap"

  # ... other configuration
}
```

| Attribute | Value | Purpose |
|-----------|-------|---------|
| `filename` | `../target/lambda/static-web-lambda/bootstrap.zip` | Path to cargo-lambda output |
| `source_code_hash` | `filebase64sha256(...)` | Triggers redeployment on code changes |
| `architectures` | `["arm64"]` | Matches the `--arm64` build flag |
| `runtime` | `"provided.al2023"` | Custom runtime for Rust binary |
| `handler` | `"bootstrap"` | Convention for custom runtimes |

### Deployment Workflow

1. **Build** — `cargo lambda build --release --arm64 --output-format zip`
2. **Plan** — `cd terraform && terraform plan` (review changes)
3. **Apply** — `cd terraform && terraform apply` (deploy to AWS)
4. **Verify** — Check the Function URL output from Terraform

## Makefile Targets

| Target | Command | Description |
|--------|---------|-------------|
| `make build-lambda` | `cargo lambda build --release --arm64 --output-format zip` | Build deployment artifact |
| `make watch` | `cargo lambda watch` | Start local emulator with hot-reload |
| `make invoke` | `cargo lambda invoke ...` | Send test event to local emulator |
| `make deploy` | `cd terraform && terraform apply` | Deploy via Terraform |
| `make build-deploy` | build-lambda + deploy | Build and deploy in one step |

## GitHub Actions CI/CD

Here's a complete GitHub Actions workflow for building and deploying the Lambda function:

```yaml
# .github/workflows/deploy.yml
name: Build and Deploy Lambda

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    permissions:
      id-token: write
      contents: read

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-lambda
        run: pip3 install cargo-lambda

      - name: Build Lambda artifact
        run: cargo lambda build --release --arm64 --output-format zip

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3

      - name: Terraform Init
        working-directory: terraform
        run: terraform init

      - name: Terraform Plan
        working-directory: terraform
        run: terraform plan -out=tfplan

      - name: Terraform Apply
        working-directory: terraform
        run: terraform apply -auto-approve tfplan
```

> **Note:** Configure AWS credentials in your workflow using `aws-actions/configure-aws-credentials` or your preferred method (OIDC, environment variables, etc.) before the Terraform steps.

### Adapting for Other CI Systems

The same approach works for any CI system. The key steps are:

1. Install Rust (stable toolchain)
2. Install cargo-lambda via `pip3 install cargo-lambda`
3. Run `cargo lambda build --release --arm64 --output-format zip`
4. Run Terraform to deploy

For example, in **GitLab CI**:

```yaml
deploy:
  image: rust:latest
  script:
    - pip3 install cargo-lambda
    - cargo lambda build --release --arm64 --output-format zip
    - cd terraform && terraform init && terraform apply -auto-approve
```

## Troubleshooting

### Build Issues

**cargo-lambda not found:**
```bash
# Reinstall
pip3 install cargo-lambda

# Or check PATH includes pip bin directory
python3 -m site --user-base  # shows install location
```

**Build fails with compilation error:**
This is a Rust source code issue, not a cargo-lambda problem. Read the compiler error and fix the source code. Running `cargo check` gives the same errors faster.

**Output ZIP not found at expected path:**
Ensure your `Cargo.toml` has the correct binary name:
```toml
[[bin]]
name = "static-web-lambda"
```
The output path uses this name: `target/lambda/<bin-name>/bootstrap.zip`

### Local Development Issues

**Port 9000 already in use:**
Another process is using the port. Stop it or find what's listening:
```bash
lsof -i :9000
```

**`cargo lambda invoke` connection refused:**
The watcher must be running first. Start it with `make watch` in another terminal.

**Hot-reload not triggering:**
Ensure your edits are in `src/` — cargo-lambda watches the source directory by default.

### Deployment Issues

**Terraform errors on missing file:**
You must build before deploying. Run `make build-lambda` first, then `make deploy`.

**Lambda function fails at runtime (wrong architecture):**
Verify you passed `--arm64` during build. The Terraform config expects ARM64 (`architectures = ["arm64"]`).

**Stale deployment (source_code_hash not changing):**
Run a clean build:
```bash
cargo clean
cargo lambda build --release --arm64 --output-format zip
```

**Lambda returns "Runtime.InvalidEntrypoint":**
The binary must be named `bootstrap`. cargo-lambda handles this automatically — if you see this error, ensure you're using the ZIP from `target/lambda/static-web-lambda/bootstrap.zip` and not a manually created package.

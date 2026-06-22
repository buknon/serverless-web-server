# Design Document

## Overview

This design describes the migration from the current Docker-based cross-compilation build process to cargo-lambda for the Static Web Lambda project. The migration simplifies the build, test, and deploy workflow by replacing Dockerfile.build, custom shell scripts, and manual packaging with cargo-lambda's native tooling. The project will target ARM64 (Graviton) on `provided.al2023` for better cost-performance.

## Architecture

### Current Architecture (Being Replaced)

```
Developer Machine
├── scripts/build-lambda.sh → Docker (Dockerfile.build) → target/release/static-web-lambda
├── scripts/build-deploy.sh → Interactive menu wrapper
├── Manual: cp binary → lambda-package/bootstrap
├── Manual: zip → lambda-deployment.zip
└── terraform apply (references lambda-deployment.zip)
```

### New Architecture (cargo-lambda)

```
Developer Machine
├── cargo lambda build --release --arm64 → target/lambda/static-web-lambda/bootstrap.zip
├── cargo lambda watch → Local Lambda emulator (development)
└── terraform apply (references target/lambda/static-web-lambda/bootstrap.zip)
```

### Component Changes

| Component | Current | After Migration |
|-----------|---------|-----------------|
| Build tool | Docker + Dockerfile.build | cargo-lambda |
| Compilation target | x86_64-unknown-linux-gnu via Docker | aarch64-unknown-linux-gnu via Zig (cargo-lambda internal) |
| Packaging | Manual shell scripts + zip | cargo-lambda (automatic) |
| Lambda runtime | provided.al2 | provided.al2023 |
| Lambda architecture | x86_64 | arm64 |
| Local development | --mode local (hyper server) | cargo lambda watch + existing --mode local |
| CI/CD | Docker-based | cargo-lambda binary install |

## Components and Interfaces

### cargo-lambda CLI

The primary build tool that replaces Docker-based compilation.

**Interface:**
- `cargo lambda build --release --arm64 --output-format zip` — produces deployment artifact
- `cargo lambda watch` — starts local Lambda emulator with hot-reload on port 9000
- `cargo lambda invoke` — sends test events to local or remote Lambda functions

**Inputs:** Rust source code, `Cargo.toml` manifest
**Outputs:** `target/lambda/static-web-lambda/bootstrap.zip` (ZIP containing ARM64 `bootstrap` binary)

### Makefile

Orchestration layer exposing common workflows as `make` targets.

**Interface:**
- `make build-lambda` — triggers cargo-lambda build
- `make watch` — starts local development emulator
- `make deploy` — runs Terraform apply
- `make build-deploy` — build then deploy in sequence

**Inputs:** Developer commands
**Outputs:** Delegates to cargo-lambda and Terraform

### Terraform Configuration (`terraform/lambda.tf`)

Infrastructure-as-code that provisions the Lambda function and references the build artifact.

**Interface:**
- `filename` attribute — path to the ZIP artifact produced by cargo-lambda
- `source_code_hash` attribute — SHA256 hash of the ZIP for change detection
- `architectures` — set to `["arm64"]`
- `runtime` — set to `"provided.al2023"`

**Inputs:** Build artifact at `target/lambda/static-web-lambda/bootstrap.zip`
**Outputs:** Deployed AWS Lambda function with Function URL

### Interaction Flow

```
Developer → Makefile → cargo-lambda CLI → bootstrap.zip → Terraform → AWS Lambda
```

## Data Models

Since this is a tooling/configuration migration with no new application data models, the relevant "data" is the build artifact structure flowing through the pipeline.

### Build Artifact Structure

```
target/lambda/static-web-lambda/bootstrap.zip
└── bootstrap          # ARM64 Linux ELF executable
```

| Field | Type | Description |
|-------|------|-------------|
| Artifact path | File path | `target/lambda/static-web-lambda/bootstrap.zip` |
| Binary name | String | Always `bootstrap` (required by Lambda custom runtime) |
| Architecture | Enum | `aarch64-unknown-linux-gnu` (ARM64) |
| Format | Archive | ZIP containing a single executable |

### Terraform Resource Attributes

| Attribute | Value | Source |
|-----------|-------|--------|
| `filename` | `${path.module}/../target/lambda/static-web-lambda/bootstrap.zip` | cargo-lambda output |
| `source_code_hash` | `filebase64sha256(...)` of the ZIP | Computed by Terraform |
| `architectures` | `["arm64"]` | Static configuration |
| `runtime` | `"provided.al2023"` | Static configuration |
| `handler` | `"bootstrap"` | Lambda custom runtime convention |

## Implementation Details

### 1. Build Tool Replacement

cargo-lambda handles cross-compilation internally using Zig as a linker, eliminating the need for Docker, platform-specific toolchains, or manual target configuration.

**Build command:**
```bash
cargo lambda build --release --arm64 --output-format zip
```

**Output path:**
```
target/lambda/static-web-lambda/bootstrap.zip
```

cargo-lambda automatically:
- Cross-compiles to the correct Linux target
- Names the binary `bootstrap`
- Creates the ZIP deployment package
- Strips debug symbols for optimal size

### 2. Terraform Updates

The `terraform/lambda.tf` file will be updated:

```hcl
resource "aws_lambda_function" "static_web_lambda" {
  # Updated to reference cargo-lambda output
  filename         = "${path.module}/../target/lambda/static-web-lambda/bootstrap.zip"
  source_code_hash = filebase64sha256("${path.module}/../target/lambda/static-web-lambda/bootstrap.zip")
  
  # Updated architecture and runtime
  architectures = ["arm64"]
  runtime       = "provided.al2023"
  
  # Handler remains the same for custom runtimes
  handler = "bootstrap"
  
  # ... rest of configuration unchanged
}
```

### 3. Makefile Updates

Replace Docker-based targets with cargo-lambda equivalents:

```makefile
# Build Lambda deployment artifact
build-lambda:
	cargo lambda build --release --arm64 --output-format zip

# Local development with hot-reload
watch:
	cargo lambda watch

# Invoke locally for testing
invoke:
	cargo lambda invoke --data-ascii '{"httpMethod": "GET", "path": "/"}'

# Deploy via Terraform
deploy:
	cd terraform && terraform apply

# Full build and deploy
build-deploy: build-lambda deploy
```

Targets to remove: `build-docker`, `build-local`, `deploy-package`, `package-zip`.

### 4. Files to Remove

- `Dockerfile.build` — no longer needed
- `scripts/build-lambda.sh` — replaced by `cargo lambda build`
- `scripts/build-deploy.sh` — replaced by Makefile targets
- `lambda-package/` directory — cargo-lambda handles packaging
- `lambda-deployment.zip` — output moves to `target/lambda/`
- `CROSS_COMPILATION_SETUP.md` — obsolete
- CodeBuild generation code within scripts — replaced by simple CI workflow

### 5. Files to Update

- `Cargo.toml` — no changes needed (cargo-lambda reads it as-is)
- `.cargo/config.toml` — remove or clear obsolete settings
- `terraform/lambda.tf` — update artifact path, architecture, runtime
- `terraform/variables.tf` — update deployment_package_path default
- `Makefile` — replace build targets
- `README.md` — full rewrite of build/deploy sections
- `docs/DEPLOYMENT.md` — full rewrite with cargo-lambda workflow
- `SCRIPT_REFERENCE.md` — remove or replace
- `.gitignore` — ensure `target/lambda/` is ignored (already covered by `target/`)
- `.dockerignore` — remove (no longer needed)

### 6. Local Development

Two local development options will exist:

1. **cargo lambda watch** (new, recommended): Starts a local Lambda emulator at `http://localhost:9000` with automatic recompilation on file changes.

2. **--mode local** (existing, retained): The existing hyper-based local HTTP server remains available for developers who prefer direct HTTP access at a configurable port.

### 7. CI/CD Updates

GitHub Actions workflow example:

```yaml
jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-lambda
        run: pip3 install cargo-lambda
      - name: Build Lambda
        run: cargo lambda build --release --arm64 --output-format zip
      - name: Deploy
        run: |
          cd terraform
          terraform init
          terraform apply -auto-approve
```

## Correctness Properties

Since this migration is primarily a tooling and configuration change (not a logic change to the application code), the correctness properties focus on verifying the migration is complete and the build output is valid.

### Property 1: Build produces valid artifact

Running `cargo lambda build --release --arm64 --output-format zip` SHALL produce a file at `target/lambda/static-web-lambda/bootstrap.zip` that contains a single executable named `bootstrap` targeting Linux ARM64.

**Validates: Requirements 1.2, 3.1, 3.2, 3.3, 3.4**

### Property 2: No Docker dependency

The build process SHALL succeed with Docker not installed or not running.

**Validates: Requirements 1.3, 2.6**

### Property 3: Terraform compatibility

The artifact path in `terraform/lambda.tf` SHALL resolve to a valid ZIP file after a successful cargo-lambda build, and `terraform plan` SHALL not produce errors related to the deployment package.

**Validates: Requirements 4.1, 4.2, 4.5**

### Property 4: Removed files do not exist

After migration, the files `Dockerfile.build`, `scripts/build-lambda.sh`, `scripts/build-deploy.sh`, `CROSS_COMPILATION_SETUP.md`, and the directory `lambda-package/` SHALL not exist in the repository.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

### Property 5: Existing application behavior preserved

The Lambda function handler logic (`src/handler.rs`, `src/lib.rs`, `src/main.rs`) SHALL remain unchanged. All existing unit tests and property-based tests SHALL continue to pass.

**Validates: Requirements 1.1**

### Property 6: Local development functional

`cargo lambda watch` SHALL start a local endpoint, and `cargo lambda invoke` SHALL return a valid HTTP response from the function.

**Validates: Requirements 7.1, 7.3**

## Error Handling

### Build Failures

| Scenario | Cause | Resolution |
|----------|-------|------------|
| `cargo lambda build` fails with compilation error | Rust source code error | Fix source code; error message from rustc is displayed directly |
| `cargo lambda build` fails with missing target | Zig linker not bundled correctly | Reinstall cargo-lambda (`pip3 install cargo-lambda` or binary release) |
| `cargo lambda build` fails with permission error | Insufficient filesystem permissions on `target/` | Check directory permissions; ensure `target/` is writable |
| Output ZIP not found at expected path | Binary name mismatch with `[[bin]]` in Cargo.toml | Ensure `Cargo.toml` has `name = "static-web-lambda"` in the `[[bin]]` section |

### Terraform Deployment Failures

| Scenario | Cause | Resolution |
|----------|-------|------------|
| `terraform plan` errors on missing file | Build was not run before `terraform apply` | Run `make build-lambda` before `make deploy` |
| `source_code_hash` mismatch causes unnecessary redeploy | Stale artifact from previous build | Run a clean build: `cargo clean && cargo lambda build --release --arm64 --output-format zip` |
| Lambda function fails at runtime | Binary architecture mismatch (x86 artifact on arm64 config) | Verify `--arm64` flag is passed during build; check `architectures = ["arm64"]` in Terraform |

### Local Development Failures

| Scenario | Cause | Resolution |
|----------|-------|------------|
| `cargo lambda watch` port conflict | Port 9000 already in use | Stop the conflicting process or configure an alternate port |
| `cargo lambda invoke` connection refused | `cargo lambda watch` not running | Start the watcher first with `make watch` |
| Hot-reload not triggering | File not in watched path | Ensure edits are in `src/` which cargo-lambda watches by default |

## Testing Strategy

- **Unit/property tests**: No changes needed — `cargo test` continues to work as before
- **Build verification**: Run `cargo lambda build --release --arm64` and verify output exists and is correct architecture
- **Integration test**: Deploy to AWS and verify Function URL returns expected HTML response
- **Regression check**: Verify all existing `cargo test` suites pass without modification

## Migration Steps (High-Level)

1. Install cargo-lambda locally
2. Verify `cargo lambda build --release --arm64 --output-format zip` succeeds
3. Update Terraform to reference new artifact path and switch to arm64/al2023
4. Update Makefile with new targets
5. Remove deprecated files (Dockerfile.build, scripts, lambda-package, etc.)
6. Update documentation (README, DEPLOYMENT.md, etc.)
7. Test full deploy cycle: build → terraform apply → verify Function URL
8. Update .gitignore and remove .dockerignore

# Requirements Document

## Introduction

This document specifies the requirements for migrating the Static Web Lambda project from a Docker-based cross-compilation build process to cargo-lambda. The current deployment pipeline uses a multi-stage Dockerfile (`Dockerfile.build`), custom shell scripts (`build-lambda.sh`, `build-deploy.sh`), and manual bootstrap binary packaging to produce Lambda-compatible ZIP artifacts. The migration replaces this with cargo-lambda, which natively handles cross-compilation, packaging, and deployment for Rust Lambda functions, dramatically simplifying the build and deploy workflow.

## Glossary

- **Build_System**: The set of tools, scripts, and configuration that compile Rust source code into a deployable Lambda artifact
- **Cargo_Lambda**: A Cargo subcommand (`cargo-lambda`) that provides native support for building, testing, and deploying Rust-based AWS Lambda functions
- **Deployment_Package**: The artifact produced by the build process that is uploaded to AWS Lambda (currently a ZIP file containing a `bootstrap` binary)
- **Bootstrap_Binary**: The executable file named `bootstrap` required by AWS Lambda custom runtimes (`provided.al2` / `provided.al2023`)
- **Terraform_Config**: The infrastructure-as-code configuration in the `terraform/` directory that provisions and manages the Lambda function and associated AWS resources
- **Build_Script**: Shell scripts in the `scripts/` directory that orchestrate the build and packaging process
- **Docker_Build**: The current build approach using `Dockerfile.build` to compile Rust code inside an Amazon Linux 2 container
- **Function_URL**: An AWS Lambda Function URL providing direct HTTPS access to the Lambda function without API Gateway

## Requirements

### Requirement 1: Install and Configure cargo-lambda

**User Story:** As a developer, I want cargo-lambda installed and configured as the build tool, so that I can build Lambda-compatible binaries without Docker or custom scripts.

#### Acceptance Criteria

1. THE Build_System SHALL use cargo-lambda as the primary tool for compiling Rust code into Lambda-compatible binaries
2. WHEN a developer runs `cargo lambda build --release`, THE Build_System SHALL produce a Lambda-compatible binary targeting the ARM64 architecture
3. THE Build_System SHALL not require Docker to be installed or running for standard Lambda builds
4. IF cargo-lambda is not installed on the developer machine, THEN THE Build_System SHALL document the installation steps clearly in the project README

### Requirement 2: Replace Docker-based Build Process

**User Story:** As a developer, I want the Docker-based build process removed and replaced with cargo-lambda commands, so that the build pipeline is simpler and faster.

#### Acceptance Criteria

1. WHEN the migration is complete, THE Build_System SHALL remove the `Dockerfile.build` file from the repository
2. WHEN the migration is complete, THE Build_System SHALL remove the `scripts/build-lambda.sh` file from the repository
3. WHEN the migration is complete, THE Build_System SHALL remove the `scripts/build-deploy.sh` file from the repository
4. WHEN the migration is complete, THE Build_System SHALL remove the `lambda-package/` directory from the repository
5. WHEN the migration is complete, THE Build_System SHALL remove the `CROSS_COMPILATION_SETUP.md` file from the repository
6. THE Build_System SHALL not depend on any Docker image or container for building Lambda deployment artifacts

### Requirement 3: Simplify Deployment Packaging

**User Story:** As a developer, I want cargo-lambda to handle packaging automatically, so that I no longer need manual ZIP creation or bootstrap binary renaming.

#### Acceptance Criteria

1. WHEN `cargo lambda build --release` completes, THE Build_System SHALL produce a deployment-ready artifact without additional packaging scripts
2. THE Build_System SHALL not require manual renaming of the compiled binary to `bootstrap`
3. THE Build_System SHALL not require manual ZIP file creation for deployment
4. WHEN `cargo lambda build --release --output-format zip` is executed, THE Build_System SHALL produce a ZIP artifact compatible with Terraform's `filename` and `source_code_hash` attributes

### Requirement 4: Update Terraform Configuration

**User Story:** As a developer, I want the Terraform configuration updated to reference the cargo-lambda output path, so that infrastructure deployments use the new build artifacts seamlessly.

#### Acceptance Criteria

1. THE Terraform_Config SHALL reference the cargo-lambda output artifact path for the `filename` attribute of the Lambda function resource
2. THE Terraform_Config SHALL compute `source_code_hash` from the cargo-lambda output artifact
3. THE Terraform_Config SHALL set the Lambda function architecture to `arm64` to match cargo-lambda's default target
4. THE Terraform_Config SHALL use the `provided.al2023` runtime to align with cargo-lambda's recommended runtime
5. WHEN `terraform apply` is executed after a cargo-lambda build, THE Terraform_Config SHALL deploy the function without manual intervention beyond the standard Terraform workflow

### Requirement 5: Update Makefile

**User Story:** As a developer, I want the Makefile updated with cargo-lambda commands, so that I can use familiar `make` targets for building and deploying.

#### Acceptance Criteria

1. THE Build_System SHALL provide a `make build-lambda` target that executes `cargo lambda build --release`
2. THE Build_System SHALL provide a `make deploy` target that executes `cargo lambda deploy` or the equivalent Terraform workflow
3. THE Build_System SHALL remove Makefile targets that reference the old Docker-based or script-based build process
4. THE Build_System SHALL provide a `make watch` target that executes `cargo lambda watch` for local development with hot-reload

### Requirement 6: Update Documentation

**User Story:** As a developer, I want all documentation updated to reflect the cargo-lambda workflow, so that setup instructions and deployment guides are accurate.

#### Acceptance Criteria

1. WHEN the migration is complete, THE Build_System SHALL update `README.md` to document the cargo-lambda build and deploy workflow
2. WHEN the migration is complete, THE Build_System SHALL update `docs/DEPLOYMENT.md` to replace all Docker-based build instructions with cargo-lambda instructions
3. WHEN the migration is complete, THE Build_System SHALL update `SCRIPT_REFERENCE.md` to reflect the simplified script set or remove it if no custom scripts remain
4. THE Build_System SHALL document prerequisite installation of cargo-lambda in the README prerequisites section
5. THE Build_System SHALL document how to test locally using `cargo lambda watch` and `cargo lambda invoke`

### Requirement 7: Local Development with cargo-lambda

**User Story:** As a developer, I want to use `cargo lambda watch` for local development, so that I can test Lambda function behavior without deploying to AWS.

#### Acceptance Criteria

1. WHEN a developer runs `cargo lambda watch`, THE Build_System SHALL start a local emulation of the Lambda runtime that accepts HTTP requests
2. WHEN source code changes are saved, THE Build_System SHALL recompile and restart the local Lambda emulator automatically
3. THE Build_System SHALL support invoking the local function using `cargo lambda invoke` with test event payloads
4. THE Build_System SHALL retain the existing `--mode local` execution mode as a secondary local development option

### Requirement 8: Update .cargo Configuration

**User Story:** As a developer, I want the `.cargo/config.toml` cleaned up to remove obsolete cross-compilation settings, so that the configuration reflects the cargo-lambda approach.

#### Acceptance Criteria

1. WHEN the migration is complete, THE Build_System SHALL remove any cross-compilation target or linker settings from `.cargo/config.toml` that were specific to the Docker-based workflow
2. THE Build_System SHALL retain any `.cargo/config.toml` settings required by cargo-lambda, if applicable
3. IF no settings remain necessary in `.cargo/config.toml`, THEN THE Build_System SHALL remove the file

### Requirement 9: CI/CD Pipeline Compatibility

**User Story:** As a developer, I want the CI/CD pipeline updated to use cargo-lambda, so that automated builds and deployments work with the new tooling.

#### Acceptance Criteria

1. THE Build_System SHALL document a GitHub Actions workflow that installs cargo-lambda and builds the Lambda artifact
2. THE Build_System SHALL document the cargo-lambda installation step suitable for CI environments (`pip install cargo-lambda` or binary download)
3. WHEN a CI pipeline runs the build step, THE Build_System SHALL produce the same deployment artifact as a local cargo-lambda build
4. THE Build_System SHALL remove the AWS CodeBuild `buildspec.yml` generation from the codebase since it is no longer needed

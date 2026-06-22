# Implementation Plan: Cargo Lambda Migration

## Overview

Migrate the Static Web Lambda project from Docker-based cross-compilation to cargo-lambda. This plan covers updating the build tooling, Terraform configuration, Makefile, documentation, and cleaning up deprecated files.

## Tasks

- [x] 1. Install and verify cargo-lambda
  - [x] 1.1 Document cargo-lambda installation in README prerequisites (`pip3 install cargo-lambda` or `brew install cargo-lambda`)
    - _Requirements: 1.4_
  - [x] 1.2 Run `cargo lambda build --release --arm64 --output-format zip` and verify output at `target/lambda/static-web-lambda/bootstrap.zip`
    - _Requirements: 1.1, 1.2, 3.1_
  - [x] 1.3 Verify the output ZIP contains a single `bootstrap` executable targeting Linux ARM64
    - _Requirements: 1.2, 3.2, 3.3_

- [x] 2. Update Terraform configuration
  - [x] 2.1 Update `terraform/lambda.tf` to set `filename` to `"${path.module}/../target/lambda/static-web-lambda/bootstrap.zip"`
    - _Requirements: 4.1_
  - [x] 2.2 Update `terraform/lambda.tf` to set `source_code_hash` to `filebase64sha256("${path.module}/../target/lambda/static-web-lambda/bootstrap.zip")`
    - _Requirements: 4.2_
  - [x] 2.3 Change `architectures` from `["x86_64"]` to `["arm64"]` in `terraform/lambda.tf`
    - _Requirements: 4.3_
  - [x] 2.4 Change `runtime` from `"provided.al2"` to `"provided.al2023"` in `terraform/lambda.tf`
    - _Requirements: 4.4_
  - [x] 2.5 Update `terraform/variables.tf` default for `deployment_package_path` to the new cargo-lambda output path
    - _Requirements: 4.1_
  - [x] 2.6 Run `terraform plan` to verify no configuration errors with the new artifact path
    - _Requirements: 4.5_

- [x] 3. Update Makefile
  - [x] 3.1 Replace `build-docker` target with `build-lambda` target running `cargo lambda build --release --arm64 --output-format zip`
    - _Requirements: 5.1, 5.3_
  - [x] 3.2 Add `watch` target running `cargo lambda watch`
    - _Requirements: 5.4_
  - [x] 3.3 Add `invoke` target running `cargo lambda invoke` with a sample GET event
    - _Requirements: 7.3_
  - [x] 3.4 Add `deploy` target running `cd terraform && terraform apply`
    - _Requirements: 5.2_
  - [x] 3.5 Add `build-deploy` target combining build-lambda and deploy
    - _Requirements: 5.1, 5.2_
  - [x] 3.6 Remove old targets: `build-docker`, `build-local`, `deploy-package`, `package-zip`
    - _Requirements: 5.3_
  - [x] 3.7 Update `help` target text to reflect new commands
    - _Requirements: 5.1_

- [x] 4. Remove deprecated files and clean up configuration
  - [x] 4.1 Delete `Dockerfile.build`
    - _Requirements: 2.1_
  - [x] 4.2 Delete `scripts/build-lambda.sh`
    - _Requirements: 2.2_
  - [x] 4.3 Delete `scripts/build-deploy.sh`
    - _Requirements: 2.3_
  - [x] 4.4 Delete `lambda-package/` directory (contains `lambda-package/bootstrap`)
    - _Requirements: 2.4_
  - [x] 4.5 Delete `lambda-deployment.zip` if present in the repository
    - _Requirements: 2.4_
  - [x] 4.6 Delete `CROSS_COMPILATION_SETUP.md`
    - _Requirements: 2.5_
  - [x] 4.7 Delete `.dockerignore`
    - _Requirements: 2.6_
  - [x] 4.8 Delete `.cargo/config.toml` (only contains a vestigial build section comment with no active settings)
    - _Requirements: 8.1, 8.3_
  - [x] 4.9 Delete `SCRIPT_REFERENCE.md` (references old scripts that no longer apply)
    - _Requirements: 6.3_
  - [x] 4.10 Delete `TESTING_SUMMARY.md` (references old build workflow)
    - _Requirements: 2.6_

- [x] 5. Checkpoint - Verify build still works after file removal
  - Ensure `cargo lambda build --release --arm64 --output-format zip` succeeds and `cargo test` passes after deprecated files are removed. Ask the user if questions arise.

- [x] 6. Update documentation
  - [x] 6.1 Rewrite README.md "Building for AWS Lambda" section to use cargo-lambda commands (`cargo lambda build --release --arm64 --output-format zip`)
    - Remove all Docker build instructions, build mode comparison table, and `build-lambda.sh` references
    - _Requirements: 6.1_
  - [x] 6.2 Update README.md "Development Workflow" subsection step 4 to use `make build-deploy` instead of `cargo build --release --target x86_64-unknown-linux-gnu`
    - _Requirements: 6.1_
  - [x] 6.3 Update README.md "Project Structure" section to remove references to deleted files (`.cargo/config.toml`, scripts, Dockerfile)
    - _Requirements: 6.1_
  - [x] 6.4 Update README.md "Deployment" section to document the new workflow: `make build-lambda` → `make deploy` (or `make build-deploy`)
    - Remove references to `build-lambda.sh`, Docker, and `lambda-deployment.zip`
    - _Requirements: 6.1_
  - [x] 6.5 Update README.md "Troubleshooting" section to remove Docker-related troubleshooting and add cargo-lambda troubleshooting (installation issues, build failures)
    - _Requirements: 6.1_
  - [x] 6.6 Add `cargo lambda watch` documentation to README.md development workflow as the recommended local testing option alongside `--mode local`
    - _Requirements: 6.5_
  - [x] 6.7 Rewrite `docs/DEPLOYMENT.md` to document cargo-lambda workflow: build command, local testing with `cargo lambda watch`/`cargo lambda invoke`, and Terraform deployment
    - Remove all Docker build methods, CodeBuild, `build-lambda.sh` references, and the old package structure
    - _Requirements: 6.2_
  - [x] 6.8 Add GitHub Actions CI/CD example to `docs/DEPLOYMENT.md` using `pip3 install cargo-lambda` and `cargo lambda build`
    - _Requirements: 9.1, 9.2, 9.3_

- [x] 7. Update .gitignore
  - [x] 7.1 Remove `*.zip` glob from `.gitignore` (the build artifact is in `target/` which is already ignored; the glob was for the old `lambda-deployment.zip`)
    - _Requirements: 2.4_
  - [x] 7.2 Remove `bootstrap` entry from `.gitignore` (was for the old `lambda-package/bootstrap`)
    - _Requirements: 2.4_
  - [x] 7.3 Remove `.dockerignore` entry from `.gitignore` (file no longer exists)
    - _Requirements: 2.6_
  - [x] 7.4 Remove `.aws-sam/` entry from `.gitignore` (not used in this project)
    - _Requirements: 2.6_

- [x] 8. Final checkpoint - Verify full pipeline
  - Ensure `cargo test` passes, `cargo lambda build --release --arm64 --output-format zip` produces the expected artifact, and `terraform plan` shows no errors. Ask the user if questions arise.

## Notes

- Tasks 1-3 are already complete (cargo-lambda installed, Terraform updated, Makefile updated)
- All existing application logic in `src/` remains unchanged — this is a tooling migration only
- The existing `--mode local` development option is retained alongside `cargo lambda watch`
- `.cargo/config.toml` currently contains only a comment with no active settings, so it can be safely deleted
- `SCRIPT_REFERENCE.md` documents `test_enhanced_logging.sh` and `demo_enhanced_logging.sh` which are unrelated to the build migration but the file itself is outdated
- The design does not include a Correctness Properties section requiring property-based tests — this is a configuration/tooling migration

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8", "4.9", "4.10"] },
    { "id": 1, "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "7.1", "7.2", "7.3", "7.4"] },
    { "id": 2, "tasks": ["6.7", "6.8"] }
  ]
}
```

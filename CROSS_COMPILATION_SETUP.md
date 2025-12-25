# Cross-Compilation Setup for AWS Lambda

This document explains how to set up cross-compilation for AWS Lambda deployment.

## Target Architecture

AWS Lambda supports two main architectures:
- **x86_64-unknown-linux-gnu**: Traditional Intel/AMD processors (widely supported)
- **aarch64-unknown-linux-gnu**: ARM64 Graviton processors (better price-performance)

## Required Toolchain Setup

### Option 1: Using rustup (Recommended)

If you have rustup installed, add the required targets:

```bash
# Add x86_64 Linux target (required for this project)
rustup target add x86_64-unknown-linux-gnu

# Optional: Add ARM64 target for Graviton support
rustup target add aarch64-unknown-linux-gnu
```

### Option 2: Cross-compilation toolchain

For cross-compilation from macOS or Windows, you'll need a Linux toolchain:

#### On macOS (using Homebrew):
```bash
# Install cross-compilation tools
brew install FiloSottile/musl-cross/musl-cross

# Or install GCC cross-compiler
brew install x86_64-linux-gnu-gcc
```

#### On Ubuntu/Debian:
```bash
# Install cross-compilation tools
sudo apt-get update
sudo apt-get install gcc-x86_64-linux-gnu
```

## Configuration Files

The project includes two configuration files for cross-compilation:

### 1. Cargo.toml
Contains target-specific linker configuration in the `[target.*]` sections.

### 2. .cargo/config.toml
Contains build configuration and environment variables for cross-compilation.

## Building for Lambda

Once the toolchain is set up, you can build for AWS Lambda:

```bash
# Build for x86_64 Lambda (traditional)
cargo build --release --target x86_64-unknown-linux-gnu

# Build for ARM64 Lambda (Graviton - better price/performance)
cargo build --release --target aarch64-unknown-linux-gnu
```

## Verification

To verify your cross-compilation setup is working:

1. Ensure you have the target installed:
   ```bash
   rustup target list --installed | grep linux-gnu
   ```

2. Try building (once main.rs exists):
   ```bash
   cargo check --target x86_64-unknown-linux-gnu
   ```

## Troubleshooting

### Common Issues:

1. **"linker not found"**: Install the appropriate cross-compilation toolchain
2. **"target not found"**: Add the target using `rustup target add`
3. **"permission denied"**: Ensure the linker executable has proper permissions

### Alternative: Using Docker

If cross-compilation is problematic, you can build inside a Linux container:

```bash
# Build using official Rust Docker image
docker run --rm -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust:1.70 cargo build --release
```

## Next Steps

After setting up cross-compilation:
1. Verify the setup works with `cargo check --target x86_64-unknown-linux-gnu`
2. Proceed to implement the main.rs file (Task 5)
3. Test the build process creates a Lambda-compatible binary
# Static Web Lambda 🦀

A simple Rust-based webserver that serves a single static HTML page on AWS Lambda.

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **AWS CLI** (optional, for deployment) - [Install AWS CLI](https://aws.amazon.com/cli/)

### Environment Setup

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Add AWS Lambda targets** (required for deployment):
   ```bash
   # Add x86_64 Linux target (required)
   rustup target add x86_64-unknown-linux-gnu
   
   # Optional: Add ARM64 target for better price/performance
   rustup target add aarch64-unknown-linux-gnu
   ```

3. **Install cross-compilation tools** (macOS):
   ```bash
   # Using Homebrew
   brew install FiloSottile/musl-cross/musl-cross
   # OR
   brew install x86_64-linux-gnu-gcc
   ```

4. **Clone and setup**:
   ```bash
   git clone <your-repo-url>
   cd static-web-lambda
   cargo build
   ```

## 🧪 Testing

This project includes comprehensive testing with both unit tests and property-based tests.

### Run All Tests
```bash
# Quick test command
make test

# Or directly with cargo
cargo test
```

### Test HTML Content
```bash
# Test HTML structure and Lambda output
make test-html

# Or run the script directly
./test_html.sh
```

### Test Types

1. **Unit Tests** (`src/tests/unit_tests.rs`)
   - Test specific functions and edge cases
   - Validate security headers and content sanitization

2. **Property-Based Tests** (`src/tests/property_tests.rs`)
   - Uses [proptest](https://github.com/proptest-rs/proptest) framework
   - Tests properties across randomly generated inputs
   - Validates security and correctness properties

3. **Integration Tests** (`src/tests/integration_tests.rs`)
   - End-to-end Lambda function testing
   - HTTP response validation

### Property-Based Testing

This project uses property-based testing to ensure robust security and correctness:

```bash
# Run property tests specifically
cargo test property_

# Run with verbose output to see generated test cases
cargo test property_ -- --nocapture
```

Property tests automatically generate hundreds of test cases to verify that security properties hold across all possible inputs.

## 🔨 Development Commands

Use the included Makefile for convenient development:

```bash
# Show all available commands
make help

# Build the project
make build

# Run all tests
make test

# Test HTML content specifically  
make test-html

# Check code without building
make check

# Clean build artifacts
make clean

# Run Lambda function locally (will timeout after 5s)
make run
```

## 🏗️ Building for AWS Lambda

### Local Development Build
```bash
cargo build
```

### Production Build for Lambda
```bash
# Build for x86_64 Lambda (traditional)
cargo build --release --target x86_64-unknown-linux-gnu

# Build for ARM64 Lambda (Graviton - better price/performance)
cargo build --release --target aarch64-unknown-linux-gnu
```

The binary will be created in `target/{target}/release/static-web-lambda`.

## 📁 Project Structure

```
static-web-lambda/
├── src/
│   ├── handler.rs          # Lambda request handler
│   ├── response.rs         # HTTP response utilities
│   ├── security.rs         # Security headers and validation
│   ├── index.html          # Static HTML content
│   ├── lib.rs             # Library root
│   ├── main.rs            # Lambda runtime entry point
│   └── tests/             # Test modules
│       ├── unit_tests.rs      # Unit tests
│       ├── property_tests.rs  # Property-based tests
│       └── integration_tests.rs # Integration tests
├── .cargo/config.toml     # Cross-compilation config
├── Cargo.toml            # Dependencies and metadata
├── Makefile              # Development commands
└── test_html.sh          # HTML content validation script
```

## 🔒 Security Features

This Lambda function includes several security measures:

- **Content Security Policy (CSP)** headers
- **Input sanitization** and validation
- **XSS protection** headers
- **HTTPS enforcement** headers
- **Content type validation**

All security features are validated through property-based tests.

## 🐛 Troubleshooting

### Common Issues

1. **Cross-compilation errors**:
   ```bash
   # Ensure targets are installed
   rustup target list --installed | grep linux-gnu
   
   # Reinstall if missing
   rustup target add x86_64-unknown-linux-gnu
   ```

2. **Test failures**:
   ```bash
   # Run tests with detailed output
   cargo test -- --nocapture
   
   # Run specific test
   cargo test test_name
   ```

3. **Property test failures**:
   - Check `proptest-regressions/` for saved failing cases
   - Property tests save counterexamples for debugging
   - Run `cargo test` again to verify fixes

### Alternative Build with Docker

If cross-compilation is problematic:

```bash
# Build using official Rust Docker image
docker run --rm -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust:1.70 cargo build --release --target x86_64-unknown-linux-gnu
```

## 📚 Dependencies

### Runtime Dependencies
- `lambda_runtime` - AWS Lambda runtime for Rust
- `lambda_http` - HTTP event handling for Lambda
- `tokio` - Async runtime
- `serde_json` - JSON serialization
- `hyper` - HTTP library
- `log` & `env_logger` - Logging
- `chrono` - Date/time handling

### Development Dependencies
- `proptest` - Property-based testing framework
- `tokio-test` - Async testing utilities

## 🚀 Deployment

After building for the target architecture, package the binary for AWS Lambda deployment:

1. Build for Lambda:
   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu
   ```

2. Package the binary (rename to `bootstrap` for Lambda):
   ```bash
   cp target/x86_64-unknown-linux-gnu/release/static-web-lambda bootstrap
   zip lambda-deployment.zip bootstrap
   ```

3. Deploy using AWS CLI, CDK, or the AWS Console.

## 📄 License

MIT License - see LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run the full test suite: `make test`
5. Submit a pull request

Make sure all tests pass, including property-based tests, before submitting.
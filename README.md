# Static Web Lambda 🦀

A simple Rust-based webserver that serves a single static HTML page on AWS Lambda.

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.83 or later) - [Install Rust](https://rustup.rs/)
- **Docker** (for Lambda builds) - [Install Docker](https://docs.docker.com/get-docker/)
- **AWS CLI** (optional, for deployment) - [Install AWS CLI](https://aws.amazon.com/cli/)

### Environment Setup

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Install Docker** (required for Lambda builds):
   - **macOS**: Download Docker Desktop from [docker.com](https://www.docker.com/products/docker-desktop)
   - **Linux**: Follow [Docker installation guide](https://docs.docker.com/engine/install/)
   - **Windows**: Download Docker Desktop from [docker.com](https://www.docker.com/products/docker-desktop)

3. **Clone and setup**:
   ```bash
   git clone <your-repo-url>
   cd static-web-lambda
   cargo build  # For local development
   ```

## 🎯 Execution Modes

This application supports two execution modes to provide flexibility for both development and production deployment:

### 🏠 Local Development Mode

Run the application as a local HTTP server for development and testing:

```bash
# Run with default settings (localhost:3000)
./target/debug/static-web-lambda --mode local

# Customize host and port
./target/debug/static-web-lambda --mode local --host 127.0.0.1 --port 8080

# Allow external connections (less secure)
./target/debug/static-web-lambda --mode local --host 0.0.0.0 --port 3000
```

**Local Mode Features:**
- 🚀 **Rapid Development**: Test changes instantly without AWS deployment
- 🔍 **Easy Debugging**: Use local debugging tools and IDE integration
- 🔒 **Same Security**: All security features work identically to Lambda mode
- 📝 **Consistent Logging**: Same structured logging format as production
- 🎯 **Identical Behavior**: Uses the exact same handler function as Lambda

**Local Mode Use Cases:**
- Development and testing new features
- Debugging issues without AWS costs
- Integration testing with other local services
- Demonstrating functionality without AWS account
- Learning Rust and serverless concepts

### ☁️ Lambda Production Mode

Run the application on AWS Lambda for production deployment:

```bash
# Lambda mode (default - used when deployed to AWS)
./target/debug/static-web-lambda --mode lambda

# Or simply (lambda is the default mode)
./target/debug/static-web-lambda
```

**Lambda Mode Features:**
- 🌐 **Serverless Scaling**: Automatic scaling based on traffic
- 💰 **Cost Efficient**: Pay only for actual requests
- 🔐 **AWS Integration**: Native CloudWatch logging and monitoring
- 🚀 **Global Distribution**: Deploy to multiple AWS regions
- 🛡️ **AWS Security**: Leverage AWS security and compliance features

**Lambda Mode Requirements:**
- Must be deployed to AWS Lambda environment
- Requires AWS Lambda environment variables
- Uses AWS Lambda Function URLs for HTTP access
- Logs automatically sent to CloudWatch

### 🔄 Mode Consistency

Both modes use the **exact same handler function** to ensure identical behavior:

| Feature | Local Mode | Lambda Mode |
|---------|------------|-------------|
| **HTTP Responses** | ✅ Identical | ✅ Identical |
| **Security Headers** | ✅ All applied | ✅ All applied |
| **Input Validation** | ✅ Same rules | ✅ Same rules |
| **Error Handling** | ✅ Same responses | ✅ Same responses |
| **Logging Format** | ✅ Structured | ✅ Structured |
| **Content Served** | ✅ Same HTML | ✅ Same HTML |

### 🛠️ Development Workflow

**Recommended development workflow:**

1. **Develop Locally**:
   ```bash
   # Start local server with logging
   RUST_LOG=info ./target/debug/static-web-lambda --mode local --port 3000
   
   # Test in browser
   open http://localhost:3000
   
   # Test with curl (watch logs in terminal)
   curl -v http://localhost:3000/
   ```

2. **Test Security Features**:
   ```bash
   # Test method validation
   curl -X POST http://localhost:3000/  # Should return 405
   
   # Test security headers
   curl -I http://localhost:3000/  # Check headers
   
   # Test path independence
   curl http://localhost:3000/any/path  # Same content
   ```

3. **Run Test Suite**:
   ```bash
   cargo test  # Verify all tests pass
   ```

4. **Deploy to Lambda**:
   ```bash
   # Build for Lambda
   cargo build --release --target x86_64-unknown-linux-gnu
   
   # Package and deploy (see Deployment section)
   ```

### 📋 Command-Line Options

```bash
# Show all available options
./target/debug/static-web-lambda --help

# Available options:
#   -m, --mode <MODE>     Execution mode: 'lambda' or 'local' [default: lambda]
#   -p, --port <PORT>     Port for local server [default: 3000]
#   -H, --host <HOST>     Host for local server [default: 127.0.0.1]
#   -h, --help           Print help information
#   -V, --version        Print version information
```

### 🚨 Important Notes

- **Lambda Mode**: Only works when deployed to AWS Lambda with proper environment variables
- **Local Mode**: Only for development - not suitable for production traffic
- **Port Conflicts**: Ensure the chosen port is not in use by other services
- **Security**: Local mode defaults to localhost (127.0.0.1) for security
- **Consistency**: Both modes produce identical HTTP responses and behavior

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

### Manual Execution Commands

```bash
# Build the project
cargo build

# Run in local development mode (with default log level)
./target/debug/static-web-lambda --mode local

# Run in local mode with info-level logging
RUST_LOG=info ./target/debug/static-web-lambda --mode local

# Run in local mode with debug-level logging (very verbose)
RUST_LOG=debug ./target/debug/static-web-lambda --mode local

# Run in local mode with custom settings and logging
RUST_LOG=info ./target/debug/static-web-lambda --mode local --host 0.0.0.0 --port 8080

# Run in Lambda mode (requires AWS Lambda environment)
./target/debug/static-web-lambda --mode lambda

# Show help and all options
./target/debug/static-web-lambda --help
```

### 📝 Logging Configuration

The application uses structured logging that outputs to **stdout** for better visibility:

```bash
# Set log level using RUST_LOG environment variable
export RUST_LOG=info    # Show info, warn, and error messages
export RUST_LOG=debug   # Show all messages (very verbose)
export RUST_LOG=warn    # Show only warnings and errors
export RUST_LOG=error   # Show only errors (default)

# Run with specific log level
RUST_LOG=info ./target/debug/static-web-lambda --mode local
```

**Log Levels Available:**
- `error` - Only error messages (default)
- `warn` - Warnings and errors
- `info` - Info, warnings, and errors (recommended for development)
- `debug` - All messages including debug info (very verbose)
- `trace` - Maximum verbosity (includes internal library logs)

**Example Log Output:**
```
[2025-12-26T13:24:00Z INFO  static_web_lambda] Starting static-web-lambda in Local mode
[2025-12-26T13:24:00Z INFO  static_web_lambda] Local development server running at http://127.0.0.1:3000
[2025-12-26T13:24:25Z INFO  static_web_lambda::handler] [REQUEST] method=GET path=/ user_agent=curl/8.7.1
[2025-12-26T13:24:25Z INFO  static_web_lambda::handler] [RESPONSE] status=200 processing_time_ms=0 path=/
```

## 🏗️ Building for AWS Lambda

### Local Development Build
```bash
cargo build
```

### Production Build for Lambda (Docker - Recommended)
```bash
# Build using Docker with Amazon Linux 2 (matches Lambda runtime exactly)
./scripts/build-lambda.sh docker

# This creates a lambda-deployment.zip file ready for deployment
```

### Alternative: Local Cross-compilation Build
```bash
# Build for x86_64 Lambda using local toolchain
./scripts/build-lambda.sh local
```

The Docker approach is recommended as it uses the same Amazon Linux 2 base image as AWS Lambda, ensuring complete glibc compatibility and avoiding runtime issues.

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

1. **Docker build errors**:
   ```bash
   # Ensure Docker is running
   docker info
   
   # Try with verbose output
   ./scripts/build-lambda.sh docker --verbose
   ```

2. **Local cross-compilation errors**:
   ```bash
   # Ensure Linux target is installed
   rustup target add x86_64-unknown-linux-gnu
   
   # Clean and retry
   ./scripts/build-lambda.sh local --clean
   ```

3. **Test failures**:
   ```bash
   # Run tests with detailed output
   cargo test -- --nocapture
   
   # Run specific test
   cargo test test_name
   ```

4. **Property test failures**:
   - Check `proptest-regressions/` for saved failing cases
   - Property tests save counterexamples for debugging
   - Run `cargo test` again to verify fixes

### Alternative Build with Docker

The project includes a Docker-based build system that uses Amazon Linux 2 (same as AWS Lambda runtime):

```bash
# Build Lambda deployment package using Docker
./scripts/build-lambda.sh docker

# This creates lambda-deployment.zip ready for deployment
```

This approach is recommended as it ensures complete compatibility with AWS Lambda runtime environment.

## 📚 Dependencies

### Runtime Dependencies
- `lambda_runtime` - AWS Lambda runtime for Rust
- `lambda_http` - HTTP event handling for Lambda
- `tokio` - Async runtime
- `serde_json` - JSON serialization
- `hyper` - HTTP library (used for local development server)
- `log` & `env_logger` - Logging
- `chrono` - Date/time handling
- `clap` - Command-line argument parsing (enables execution modes)

### Development Dependencies
- `proptest` - Property-based testing framework
- `tokio-test` - Async testing utilities

## 🚀 Deployment

The project includes automated build scripts for creating Lambda deployment packages:

1. **Build Lambda package using Docker** (recommended):
   ```bash
   ./scripts/build-lambda.sh docker
   ```

2. **Alternative: Build using local cross-compilation**:
   ```bash
   ./scripts/build-lambda.sh local
   ```

3. **Deploy the generated package**:
   ```bash
   # The build script creates lambda-deployment.zip
   # Deploy using AWS CLI, CDK, Terraform, or AWS Console
   ```

The Docker build uses Amazon Linux 2 (same as Lambda runtime) to ensure complete compatibility.

## 📄 License

MIT License - see LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run the full test suite: `make test`
5. Submit a pull request

Make sure all tests pass, including property-based tests, before submitting.
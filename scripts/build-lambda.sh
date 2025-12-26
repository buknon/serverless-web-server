#!/bin/bash

# Build script for AWS Lambda deployment
# This script handles native compilation and deployment preparation for the static web Lambda function
# Supports multiple build methods: Docker, local cross-compilation, and AWS CodeBuild

set -euo pipefail

# Configuration
PROJECT_NAME="static-web-lambda"
TARGET_DIR="target"
LAMBDA_DIR="lambda-package"
BOOTSTRAP_NAME="bootstrap"
PACKAGE_NAME="lambda-deployment.zip"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
AWS Lambda Build Script for Static Web Lambda

USAGE:
    $0 [OPTIONS] [BUILD_METHOD]

BUILD METHODS:
    docker          Build using Docker (recommended for cross-platform)
    local           Build using local cross-compilation toolchain
    native          Build using native compilation (for testing only)
    codebuild       Generate AWS CodeBuild configuration

OPTIONS:
    -h, --help      Show this help message
    -v, --verbose   Enable verbose output
    -c, --clean     Clean build artifacts before building
    --validate      Validate build environment and dependencies
    --package-only  Only create deployment package (skip compilation)

EXAMPLES:
    $0 docker                    # Build using Docker
    $0 local --clean             # Clean build using local toolchain
    $0 --validate                # Check build environment
    $0 codebuild                 # Generate CodeBuild configuration

ENVIRONMENT VARIABLES:
    DOCKER_IMAGE    Custom Docker image for building (default: rust:1.83)
    TARGET_ARCH     Target architecture (default: x86_64-unknown-linux-gnu)
    CARGO_FLAGS     Additional cargo build flags

For more information, see the deployment documentation in docs/DEPLOYMENT.md
EOF
}

# Validate build environment
validate_environment() {
    log_info "Validating build environment..."
    
    local errors=0
    
    # Check if we're in the right directory
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "Cargo.toml not found. Please run this script from the project root."
        ((errors++))
    fi
    
    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed. Please install Rust and Cargo."
        ((errors++))
    fi
    
    # Check Docker availability for Docker builds
    if ! command -v docker &> /dev/null; then
        log_warning "Docker is not available. Docker-based builds will not work."
    else
        if ! docker info &> /dev/null; then
            log_warning "Docker daemon is not running. Docker-based builds will not work."
        fi
    fi
    
    # Check for required files
    local required_files=("src/main.rs" "src/lib.rs")
    for file in "${required_files[@]}"; do
        if [[ ! -f "$file" ]]; then
            log_error "Required file $file not found."
            ((errors++))
        fi
    done
    
    if [[ $errors -eq 0 ]]; then
        log_success "Build environment validation passed."
        return 0
    else
        log_error "Build environment validation failed with $errors errors."
        return 1
    fi
}

# Clean build artifacts
clean_build() {
    log_info "Cleaning build artifacts..."
    
    # Clean cargo build artifacts
    if [[ -d "$TARGET_DIR" ]]; then
        rm -rf "$TARGET_DIR"
        log_info "Removed $TARGET_DIR directory"
    fi
    
    # Clean lambda package directory
    if [[ -d "$LAMBDA_DIR" ]]; then
        rm -rf "$LAMBDA_DIR"
        log_info "Removed $LAMBDA_DIR directory"
    fi
    
    # Clean deployment package
    if [[ -f "$PACKAGE_NAME" ]]; then
        rm -f "$PACKAGE_NAME"
        log_info "Removed $PACKAGE_NAME"
    fi
    
    log_success "Build artifacts cleaned."
}

# Build using Docker
build_docker() {
    log_info "Building using Docker..."
    
    local target_arch="${TARGET_ARCH:-x86_64-unknown-linux-gnu}"
    local cargo_flags="${CARGO_FLAGS:-}"
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker to use this build method."
        return 1
    fi
    
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker."
        return 1
    fi
    
    log_info "Using custom Dockerfile.build for AWS Lambda compatibility"
    log_info "Target architecture: $target_arch"
    
    # Build the Docker image from our custom Dockerfile
    log_info "Building Docker image..."
    if ! docker build -f Dockerfile.build -t lambda-rust-builder .; then
        log_error "Failed to build Docker image"
        return 1
    fi
    
    # Run the build using our custom image
    log_info "Running build in Docker container..."
    if docker run --rm --platform linux/amd64 \
        -v "$PWD":/usr/src/app \
        -w /usr/src/app \
        lambda-rust-builder; then
        
        log_success "Docker build completed successfully."
        
        # Copy the binary to expected location for compatibility
        local binary_path="$TARGET_DIR/release/$PROJECT_NAME"
        if [[ -f "$binary_path" ]]; then
            log_info "Binary built at: $binary_path"
            return 0
        else
            log_error "Binary not found at expected location: $binary_path"
            return 1
        fi
    else
        log_error "Docker build failed."
        return 1
    fi
}

# Build using local cross-compilation
build_local() {
    log_info "Building using local cross-compilation..."
    
    local target_arch="${TARGET_ARCH:-x86_64-unknown-linux-gnu}"
    local cargo_flags="${CARGO_FLAGS:-}"
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "$target_arch"; then
        log_info "Installing target $target_arch..."
        if ! rustup target add "$target_arch"; then
            log_error "Failed to install target $target_arch"
            return 1
        fi
    fi
    
    log_info "Building for target: $target_arch"
    
    # Build the project
    if cargo build --release --target "$target_arch" $cargo_flags; then
        log_success "Local cross-compilation build completed successfully."
        
        # Verify binary exists
        local binary_path="$TARGET_DIR/$target_arch/release/$PROJECT_NAME"
        if [[ -f "$binary_path" ]]; then
            log_info "Binary built at: $binary_path"
            return 0
        else
            log_error "Binary not found at expected location: $binary_path"
            return 1
        fi
    else
        log_error "Local cross-compilation build failed."
        return 1
    fi
}

# Build using native compilation (for testing only)
build_native() {
    log_warning "Building using native compilation - this is for testing only!"
    log_warning "Native builds will NOT work on AWS Lambda unless built on Amazon Linux."
    
    local cargo_flags="${CARGO_FLAGS:-}"
    
    # Build the project
    if cargo build --release $cargo_flags; then
        log_success "Native build completed successfully."
        
        # Verify binary exists
        local binary_path="$TARGET_DIR/release/$PROJECT_NAME"
        if [[ -f "$binary_path" ]]; then
            log_info "Binary built at: $binary_path"
            return 0
        else
            log_error "Binary not found at expected location: $binary_path"
            return 1
        fi
    else
        log_error "Native build failed."
        return 1
    fi
}

# Create deployment package
create_package() {
    log_info "Creating deployment package..."
    
    # Determine binary path based on build method
    local binary_path=""
    local target_arch="${TARGET_ARCH:-x86_64-unknown-linux-gnu}"
    
    # Try different possible binary locations
    if [[ -f "$TARGET_DIR/$target_arch/release/$PROJECT_NAME" ]]; then
        binary_path="$TARGET_DIR/$target_arch/release/$PROJECT_NAME"
    elif [[ -f "$TARGET_DIR/release/$PROJECT_NAME" ]]; then
        binary_path="$TARGET_DIR/release/$PROJECT_NAME"
    else
        log_error "No binary found. Please build the project first."
        return 1
    fi
    
    log_info "Using binary: $binary_path"
    
    # Create lambda package directory
    mkdir -p "$LAMBDA_DIR"
    
    # Copy binary and rename to bootstrap (required by Lambda)
    cp "$binary_path" "$LAMBDA_DIR/$BOOTSTRAP_NAME"
    
    # Set executable permissions
    chmod +x "$LAMBDA_DIR/$BOOTSTRAP_NAME"
    
    # Validate the bootstrap binary
    if [[ ! -x "$LAMBDA_DIR/$BOOTSTRAP_NAME" ]]; then
        log_error "Bootstrap binary is not executable."
        return 1
    fi
    
    # Get binary size for validation
    local binary_size=$(stat -f%z "$LAMBDA_DIR/$BOOTSTRAP_NAME" 2>/dev/null || stat -c%s "$LAMBDA_DIR/$BOOTSTRAP_NAME" 2>/dev/null || echo "unknown")
    log_info "Bootstrap binary size: $binary_size bytes"
    
    # Create ZIP package
    log_info "Creating ZIP package: $PACKAGE_NAME"
    
    # Change to lambda directory to avoid including directory structure in ZIP
    (cd "$LAMBDA_DIR" && zip -r "../$PACKAGE_NAME" "$BOOTSTRAP_NAME")
    
    if [[ -f "$PACKAGE_NAME" ]]; then
        local package_size=$(stat -f%z "$PACKAGE_NAME" 2>/dev/null || stat -c%s "$PACKAGE_NAME" 2>/dev/null || echo "unknown")
        log_success "Deployment package created: $PACKAGE_NAME ($package_size bytes)"
        
        # Validate ZIP contents
        log_info "ZIP package contents:"
        unzip -l "$PACKAGE_NAME"
        
        return 0
    else
        log_error "Failed to create deployment package."
        return 1
    fi
}

# Generate AWS CodeBuild configuration
generate_codebuild() {
    log_info "Generating AWS CodeBuild configuration..."
    
    local buildspec_file="buildspec.yml"
    
    cat > "$buildspec_file" << 'EOF'
# AWS CodeBuild buildspec for Static Web Lambda
# This configuration builds the Rust Lambda function in a Linux environment
# and creates a deployment package compatible with AWS Lambda

version: 0.2

phases:
  install:
    runtime-versions:
      rust: 1.70
    commands:
      - echo "Installing Rust toolchain and dependencies..."
      - rustup target add x86_64-unknown-linux-gnu
      - yum install -y zip
      
  pre_build:
    commands:
      - echo "Pre-build phase - validating environment..."
      - rustc --version
      - cargo --version
      - echo "Current directory contents:"
      - ls -la
      
  build:
    commands:
      - echo "Building Lambda function..."
      - cargo build --release --target x86_64-unknown-linux-gnu
      - echo "Build completed successfully"
      
  post_build:
    commands:
      - echo "Creating deployment package..."
      - mkdir -p lambda-package
      - cp target/x86_64-unknown-linux-gnu/release/static-web-lambda lambda-package/bootstrap
      - chmod +x lambda-package/bootstrap
      - cd lambda-package && zip -r ../lambda-deployment.zip bootstrap
      - cd ..
      - echo "Deployment package created"
      - ls -la lambda-deployment.zip
      - echo "Package contents:"
      - unzip -l lambda-deployment.zip

artifacts:
  files:
    - lambda-deployment.zip
  name: static-web-lambda-$(date +%Y-%m-%d-%H-%M-%S)
  
cache:
  paths:
    - '/root/.cargo/**/*'
    - 'target/**/*'
EOF

    log_success "AWS CodeBuild configuration created: $buildspec_file"
    
    # Also create a simple CodeBuild project template
    local template_file="codebuild-project-template.json"
    
    cat > "$template_file" << 'EOF'
{
  "name": "static-web-lambda-build",
  "description": "Build project for Static Web Lambda Rust function",
  "source": {
    "type": "GITHUB",
    "location": "https://github.com/your-username/static-web-lambda.git",
    "buildspec": "buildspec.yml"
  },
  "artifacts": {
    "type": "S3",
    "location": "your-build-artifacts-bucket/static-web-lambda"
  },
  "environment": {
    "type": "LINUX_CONTAINER",
    "image": "aws/codebuild/amazonlinux2-x86_64-standard:3.0",
    "computeType": "BUILD_GENERAL1_SMALL"
  },
  "serviceRole": "arn:aws:iam::YOUR_ACCOUNT:role/service-role/codebuild-static-web-lambda-service-role"
}
EOF

    log_info "CodeBuild project template created: $template_file"
    log_info "Update the template with your specific GitHub repository and S3 bucket information."
    
    return 0
}

# Main function
main() {
    local build_method=""
    local verbose=false
    local clean=false
    local validate_only=false
    local package_only=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--verbose)
                verbose=true
                set -x
                shift
                ;;
            -c|--clean)
                clean=true
                shift
                ;;
            --validate)
                validate_only=true
                shift
                ;;
            --package-only)
                package_only=true
                shift
                ;;
            docker|local|native|codebuild)
                build_method="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Validate environment first
    if ! validate_environment; then
        exit 1
    fi
    
    if [[ "$validate_only" == true ]]; then
        log_success "Environment validation completed successfully."
        exit 0
    fi
    
    # Clean if requested
    if [[ "$clean" == true ]]; then
        clean_build
    fi
    
    # Handle package-only mode
    if [[ "$package_only" == true ]]; then
        if create_package; then
            log_success "Deployment package created successfully."
            exit 0
        else
            log_error "Failed to create deployment package."
            exit 1
        fi
    fi
    
    # Default to docker if no method specified
    if [[ -z "$build_method" ]]; then
        log_info "No build method specified, defaulting to Docker build."
        build_method="docker"
    fi
    
    # Execute build method
    case "$build_method" in
        docker)
            if build_docker && create_package; then
                log_success "Docker build and packaging completed successfully."
            else
                log_error "Docker build or packaging failed."
                exit 1
            fi
            ;;
        local)
            if build_local && create_package; then
                log_success "Local build and packaging completed successfully."
            else
                log_error "Local build or packaging failed."
                exit 1
            fi
            ;;
        native)
            if build_native && create_package; then
                log_success "Native build and packaging completed successfully."
                log_warning "Remember: Native builds may not work on AWS Lambda!"
            else
                log_error "Native build or packaging failed."
                exit 1
            fi
            ;;
        codebuild)
            if generate_codebuild; then
                log_success "AWS CodeBuild configuration generated successfully."
            else
                log_error "Failed to generate CodeBuild configuration."
                exit 1
            fi
            ;;
        *)
            log_error "Unknown build method: $build_method"
            show_help
            exit 1
            ;;
    esac
    
    log_success "Build process completed successfully!"
}

# Run main function with all arguments
main "$@"
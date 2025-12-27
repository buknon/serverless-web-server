# Makefile for Static Web Lambda project
# This provides convenient commands for development and testing

.PHONY: help build test test-html run clean check build-lambda build-docker build-local deploy-package package-zip docs docs-open

# Default target - show help
help:
	@echo "🦀 Static Web Lambda - Available Commands:"
	@echo "=========================================="
	@echo "Development Commands:"
	@echo "  make build      - Build the project locally"
	@echo "  make test       - Run all tests"
	@echo "  make test-html  - Test HTML content specifically"
	@echo "  make run        - Run the Lambda function locally (will start and stop)"
	@echo "  make check      - Check code without building"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make docs       - Generate documentation"
	@echo "  make docs-open  - Generate and open documentation in browser"
	@echo ""
	@echo "Deployment Commands:"
	@echo "  make build-lambda    - Interactive deployment build menu"
	@echo "  make build-docker    - Build Lambda package using Docker"
	@echo "  make build-local     - Build Lambda package using local cross-compilation"
	@echo "  make deploy-package  - Create deployment package from existing binary"
	@echo "  make package-zip     - Create ZIP package from bootstrap executable"
	@echo ""
	@echo "🧪 After making changes, run: make test-html"
	@echo "🚀 For deployment, run: make build-lambda"

# Build the project
build:
	@echo "🔨 Building project..."
	cargo build

# Run all tests
test:
	@echo "🧪 Running all tests..."
	cargo test

# Test HTML content specifically
test-html:
	@echo "🌐 Testing HTML content..."
	@./test_html.sh

# Run the Lambda function (starts and stops quickly since it's for Lambda)
run:
	@echo "🚀 Starting Lambda function..."
	@echo "⚠️  Note: This is a Lambda function, so it will start the runtime and wait for events."
	@echo "   Press Ctrl+C to stop, or it will timeout after a few seconds."
	@timeout 5s cargo run || echo "✅ Lambda runtime started successfully (timed out as expected)"

# Check code without building
check:
	@echo "🔍 Checking code..."
	cargo check

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Deployment build commands

# Interactive deployment build menu
build-lambda:
	@echo "🚀 Starting interactive deployment build..."
	@./scripts/build-deploy.sh

# Build Lambda package using Docker (recommended)
build-docker:
	@echo "🐳 Building Lambda package using Docker..."
	@./scripts/build-lambda.sh docker

# Build Lambda package using local cross-compilation
build-local:
	@echo "🔧 Building Lambda package using local cross-compilation..."
	@./scripts/build-lambda.sh local

# Create deployment package from existing binary
deploy-package:
	@echo "📦 Creating deployment package..."
	@./scripts/build-lambda.sh --package-only

# Create ZIP package from bootstrap executable
package-zip:
	@echo "📦 Creating ZIP package from bootstrap executable..."
	@./scripts/package-lambda.sh

docs:
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --document-private-items

docs-open:
	@echo "📚 Generating and opening documentation..."
	cargo doc --no-deps --document-private-items --open

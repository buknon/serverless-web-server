# Makefile for Static Web Lambda project
# This provides convenient commands for development and testing

.PHONY: help build test test-html run clean check build-lambda watch invoke deploy build-deploy docs docs-open

# Default target - show help
help:
	@echo "🦀 Static Web Lambda - Available Commands:"
	@echo "=========================================="
	@echo "Development Commands:"
	@echo "  make build      - Build the project locally"
	@echo "  make test       - Run all tests"
	@echo "  make test-html  - Test HTML content specifically"
	@echo "  make run        - Run the Lambda function locally (will start and stop)"
	@echo "  make watch      - Run cargo lambda watch for local development (hot-reload)"
	@echo "  make invoke     - Invoke local Lambda for testing (requires watch running)"
	@echo "  make check      - Check code without building"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make docs       - Generate documentation"
	@echo "  make docs-open  - Generate and open documentation in browser"
	@echo ""
	@echo "Deployment Commands:"
	@echo "  make build-lambda  - Build Lambda artifact with cargo-lambda (ARM64)"
	@echo "  make deploy        - Deploy via Terraform"
	@echo "  make build-deploy  - Build Lambda artifact and deploy via Terraform"
	@echo ""
	@echo "🧪 After making changes, run: make test-html"
	@echo "🚀 For deployment, run: make build-deploy"

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

# Build Lambda deployment artifact
build-lambda:
	cargo lambda build --release --arm64 --output-format zip

# Local development with hot-reload
watch:
	cargo lambda watch

# Invoke locally for testing (requires `cargo lambda watch` running in another terminal)
invoke:
	cargo lambda invoke static-web-lambda --data-ascii '{"httpMethod": "GET", "path": "/", "requestContext": {"http": {"method": "GET", "path": "/"}}}'

# Deploy via Terraform
deploy:
	cd terraform && terraform apply

# Full build and deploy
build-deploy: build-lambda deploy

docs:
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --document-private-items

docs-open:
	@echo "📚 Generating and opening documentation..."
	cargo doc --no-deps --document-private-items --open

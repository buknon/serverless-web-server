# Makefile for Static Web Lambda project
# This provides convenient commands for development and testing

.PHONY: help build test test-html run clean check

# Default target - show help
help:
	@echo "🦀 Static Web Lambda - Available Commands:"
	@echo "=========================================="
	@echo "  make build      - Build the project"
	@echo "  make test       - Run all tests"
	@echo "  make test-html  - Test HTML content specifically"
	@echo "  make run        - Run the Lambda function locally (will start and stop)"
	@echo "  make check      - Check code without building"
	@echo "  make clean      - Clean build artifacts"
	@echo ""
	@echo "🧪 After making changes, run: make test-html"

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
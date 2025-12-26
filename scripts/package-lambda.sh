#!/bin/bash

# ZIP packaging script for AWS Lambda deployment
# This script creates a ZIP file containing the bootstrap executable
# and validates the ZIP file structure for Lambda compatibility

set -euo pipefail

# Configuration
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

# Validate bootstrap executable exists and is executable
validate_bootstrap() {
    log_info "Validating bootstrap executable..."
    
    local bootstrap_path="$LAMBDA_DIR/$BOOTSTRAP_NAME"
    
    # Check if bootstrap file exists
    if [[ ! -f "$bootstrap_path" ]]; then
        log_error "Bootstrap executable not found at: $bootstrap_path"
        log_error "Please build the project first using: make build-docker or make build-local"
        return 1
    fi
    
    # Check if bootstrap is executable
    if [[ ! -x "$bootstrap_path" ]]; then
        log_error "Bootstrap file exists but is not executable: $bootstrap_path"
        log_info "Attempting to fix permissions..."
        chmod +x "$bootstrap_path"
        
        if [[ ! -x "$bootstrap_path" ]]; then
            log_error "Failed to make bootstrap executable"
            return 1
        fi
        
        log_success "Fixed bootstrap executable permissions"
    fi
    
    # Get and display bootstrap file information
    local file_size=$(stat -f%z "$bootstrap_path" 2>/dev/null || stat -c%s "$bootstrap_path" 2>/dev/null || echo "unknown")
    local file_type=$(file "$bootstrap_path" 2>/dev/null || echo "unknown")
    
    log_info "Bootstrap executable details:"
    log_info "  Path: $bootstrap_path"
    log_info "  Size: $file_size bytes"
    log_info "  Type: $file_type"
    
    # Validate file size (Lambda has a 50MB uncompressed limit)
    if [[ "$file_size" != "unknown" && "$file_size" -gt 52428800 ]]; then
        log_warning "Bootstrap executable is larger than 50MB, which may cause Lambda deployment issues"
    fi
    
    log_success "Bootstrap executable validation passed"
    return 0
}

# Create ZIP package
create_zip_package() {
    log_info "Creating ZIP deployment package..."
    
    # Remove existing package if it exists
    if [[ -f "$PACKAGE_NAME" ]]; then
        log_info "Removing existing package: $PACKAGE_NAME"
        rm -f "$PACKAGE_NAME"
    fi
    
    # Verify zip command is available
    if ! command -v zip &> /dev/null; then
        log_error "zip command not found. Please install zip utility."
        return 1
    fi
    
    # Create ZIP package
    # Change to lambda directory to avoid including directory structure in ZIP
    log_info "Packaging bootstrap executable into ZIP file..."
    
    if (cd "$LAMBDA_DIR" && zip -r "../$PACKAGE_NAME" "$BOOTSTRAP_NAME"); then
        log_success "ZIP package created successfully: $PACKAGE_NAME"
    else
        log_error "Failed to create ZIP package"
        return 1
    fi
    
    return 0
}

# Validate ZIP file structure
validate_zip_structure() {
    log_info "Validating ZIP file structure..."
    
    # Check if ZIP file exists
    if [[ ! -f "$PACKAGE_NAME" ]]; then
        log_error "ZIP package not found: $PACKAGE_NAME"
        return 1
    fi
    
    # Get ZIP file size
    local zip_size=$(stat -f%z "$PACKAGE_NAME" 2>/dev/null || stat -c%s "$PACKAGE_NAME" 2>/dev/null || echo "unknown")
    log_info "ZIP package size: $zip_size bytes"
    
    # Validate ZIP file integrity
    if ! zip -T "$PACKAGE_NAME" &> /dev/null; then
        log_error "ZIP file integrity check failed"
        return 1
    fi
    
    log_success "ZIP file integrity check passed"
    
    # List ZIP contents
    log_info "ZIP package contents:"
    if unzip -l "$PACKAGE_NAME"; then
        log_success "ZIP contents listed successfully"
    else
        log_error "Failed to list ZIP contents"
        return 1
    fi
    
    # Validate that bootstrap is in the root of the ZIP
    local zip_contents=$(unzip -l "$PACKAGE_NAME" | grep -E "^\s*[0-9]+.*bootstrap$" || true)
    if [[ -z "$zip_contents" ]]; then
        log_error "Bootstrap executable not found in ZIP root directory"
        log_error "Lambda requires the bootstrap executable to be in the ZIP root"
        return 1
    fi
    
    log_success "Bootstrap executable found in ZIP root directory"
    
    # Check for any unexpected files
    local file_count=$(unzip -l "$PACKAGE_NAME" | grep -E "^\s*[0-9]+\s+[0-9-]+\s+[0-9:]+\s+" | wc -l | tr -d ' ')
    if [[ "$file_count" -ne 1 ]]; then
        log_warning "ZIP contains $file_count files, expected 1 (bootstrap only)"
        log_info "Additional files may cause deployment issues"
    else
        log_success "ZIP contains exactly 1 file as expected"
    fi
    
    # Validate ZIP size (Lambda has a 50MB compressed limit)
    if [[ "$zip_size" != "unknown" && "$zip_size" -gt 52428800 ]]; then
        log_warning "ZIP package is larger than 50MB, which exceeds Lambda deployment limit"
        return 1
    fi
    
    log_success "ZIP file structure validation passed"
    return 0
}

# Test ZIP extraction (additional validation)
test_zip_extraction() {
    log_info "Testing ZIP extraction..."
    
    local test_dir="test-extraction"
    
    # Create temporary test directory
    if [[ -d "$test_dir" ]]; then
        rm -rf "$test_dir"
    fi
    mkdir -p "$test_dir"
    
    # Extract ZIP to test directory
    if (cd "$test_dir" && unzip -q "../$PACKAGE_NAME"); then
        log_success "ZIP extraction test passed"
        
        # Verify extracted bootstrap
        if [[ -f "$test_dir/$BOOTSTRAP_NAME" && -x "$test_dir/$BOOTSTRAP_NAME" ]]; then
            log_success "Extracted bootstrap is executable"
        else
            log_error "Extracted bootstrap is not executable"
            rm -rf "$test_dir"
            return 1
        fi
        
        # Clean up test directory
        rm -rf "$test_dir"
        log_info "Test extraction cleanup completed"
    else
        log_error "ZIP extraction test failed"
        rm -rf "$test_dir"
        return 1
    fi
    
    return 0
}

# Display package information
display_package_info() {
    log_info "Deployment package information:"
    
    if [[ -f "$PACKAGE_NAME" ]]; then
        local zip_size=$(stat -f%z "$PACKAGE_NAME" 2>/dev/null || stat -c%s "$PACKAGE_NAME" 2>/dev/null || echo "unknown")
        local zip_size_mb=$(echo "scale=2; $zip_size / 1024 / 1024" | bc 2>/dev/null || echo "unknown")
        
        echo "  📦 Package: $PACKAGE_NAME"
        echo "  📏 Size: $zip_size bytes ($zip_size_mb MB)"
        echo "  📁 Contents: bootstrap executable"
        echo "  ✅ Ready for Lambda deployment"
        
        # Show deployment instructions
        echo ""
        log_info "Deployment options:"
        echo "  1. Terraform: Place this ZIP in your terraform directory and run 'terraform apply'"
        echo "  2. AWS CLI: aws lambda update-function-code --function-name your-function --zip-file fileb://$PACKAGE_NAME"
        echo "  3. AWS Console: Upload this ZIP file through the Lambda console"
    else
        log_error "Package file not found: $PACKAGE_NAME"
        return 1
    fi
}

# Help function
show_help() {
    cat << EOF
AWS Lambda ZIP Packaging Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help      Show this help message
    -v, --verbose   Enable verbose output
    --validate-only Validate existing ZIP without recreating
    --test-only     Only test ZIP extraction without validation

DESCRIPTION:
    This script creates a ZIP deployment package for AWS Lambda containing
    the bootstrap executable. It validates the ZIP structure and ensures
    compatibility with Lambda deployment requirements.

REQUIREMENTS:
    - Bootstrap executable must exist at: $LAMBDA_DIR/$BOOTSTRAP_NAME
    - zip utility must be installed
    - Bootstrap must be executable

EXAMPLES:
    $0                    # Create and validate ZIP package
    $0 --validate-only    # Only validate existing ZIP
    $0 --verbose          # Enable verbose output

The script performs the following validations:
    - Bootstrap executable exists and is executable
    - ZIP file is created successfully
    - ZIP file integrity is verified
    - Bootstrap is in ZIP root directory
    - ZIP contains only expected files
    - ZIP size is within Lambda limits
    - ZIP extraction works correctly

EOF
}

# Main function
main() {
    local verbose=false
    local validate_only=false
    local test_only=false
    
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
            --validate-only)
                validate_only=true
                shift
                ;;
            --test-only)
                test_only=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    log_info "Starting Lambda ZIP packaging process..."
    
    # Handle test-only mode
    if [[ "$test_only" == true ]]; then
        if test_zip_extraction; then
            log_success "ZIP extraction test completed successfully"
            exit 0
        else
            log_error "ZIP extraction test failed"
            exit 1
        fi
    fi
    
    # Handle validate-only mode
    if [[ "$validate_only" == true ]]; then
        if validate_zip_structure && test_zip_extraction; then
            display_package_info
            log_success "ZIP validation completed successfully"
            exit 0
        else
            log_error "ZIP validation failed"
            exit 1
        fi
    fi
    
    # Full packaging process
    local success=true
    
    # Step 1: Validate bootstrap executable
    if ! validate_bootstrap; then
        success=false
    fi
    
    # Step 2: Create ZIP package
    if [[ "$success" == true ]] && ! create_zip_package; then
        success=false
    fi
    
    # Step 3: Validate ZIP structure
    if [[ "$success" == true ]] && ! validate_zip_structure; then
        success=false
    fi
    
    # Step 4: Test ZIP extraction
    if [[ "$success" == true ]] && ! test_zip_extraction; then
        success=false
    fi
    
    # Step 5: Display package information
    if [[ "$success" == true ]]; then
        display_package_info
        log_success "Lambda ZIP packaging completed successfully!"
        echo ""
        echo "🚀 Your Lambda deployment package is ready!"
        echo "📦 Package: $PACKAGE_NAME"
        echo "📋 Next step: Deploy using Terraform or AWS CLI"
    else
        log_error "Lambda ZIP packaging failed!"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
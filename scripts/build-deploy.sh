#!/bin/bash

# Simple deployment build wrapper script
# This script provides a simplified interface for common build operations

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🦀 Static Web Lambda - Deployment Builder${NC}"
echo "=============================================="

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_SCRIPT="$SCRIPT_DIR/build-lambda.sh"

# Check if build script exists
if [[ ! -f "$BUILD_SCRIPT" ]]; then
    echo "❌ Build script not found at: $BUILD_SCRIPT"
    exit 1
fi

# Simple menu for build options
echo ""
echo "Choose your build method:"
echo "1) Docker build (recommended - works on all platforms)"
echo "2) Local cross-compilation (faster, requires setup)"
echo "3) Native build (testing only - won't work on Lambda)"
echo "4) Generate AWS CodeBuild configuration"
echo "5) Validate build environment only"
echo "6) Clean build artifacts"
echo ""

read -p "Enter your choice (1-6): " choice

case $choice in
    1)
        echo -e "${GREEN}🐳 Starting Docker build...${NC}"
        "$BUILD_SCRIPT" docker
        ;;
    2)
        echo -e "${GREEN}🔧 Starting local cross-compilation build...${NC}"
        "$BUILD_SCRIPT" local
        ;;
    3)
        echo -e "${YELLOW}⚠️  Starting native build (testing only)...${NC}"
        "$BUILD_SCRIPT" native
        ;;
    4)
        echo -e "${GREEN}☁️  Generating AWS CodeBuild configuration...${NC}"
        "$BUILD_SCRIPT" codebuild
        ;;
    5)
        echo -e "${BLUE}🔍 Validating build environment...${NC}"
        "$BUILD_SCRIPT" --validate
        ;;
    6)
        echo -e "${BLUE}🧹 Cleaning build artifacts...${NC}"
        "$BUILD_SCRIPT" docker --clean
        echo "Build artifacts cleaned."
        ;;
    *)
        echo "❌ Invalid choice. Please run the script again and choose 1-6."
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}✅ Operation completed!${NC}"

# Show next steps based on what was built
if [[ $choice -eq 1 || $choice -eq 2 || $choice -eq 3 ]]; then
    echo ""
    echo "📦 Next steps:"
    echo "1. Your deployment package is ready: lambda-deployment.zip"
    echo "2. Deploy with Terraform: cd terraform && terraform apply"
    echo "3. Or upload directly to AWS Lambda console"
    echo ""
    echo "📋 Package information:"
    if [[ -f "lambda-deployment.zip" ]]; then
        ls -lh lambda-deployment.zip
    fi
elif [[ $choice -eq 4 ]]; then
    echo ""
    echo "📋 CodeBuild files created:"
    echo "- buildspec.yml (CodeBuild build specification)"
    echo "- codebuild-project-template.json (CodeBuild project template)"
    echo ""
    echo "📖 Next steps:"
    echo "1. Update codebuild-project-template.json with your GitHub repo and S3 bucket"
    echo "2. Create CodeBuild project: aws codebuild create-project --cli-input-json file://codebuild-project-template.json"
    echo "3. Start build: aws codebuild start-build --project-name static-web-lambda-build"
fi
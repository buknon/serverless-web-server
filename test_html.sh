#!/bin/bash

# Test script to verify HTML content after cargo run
# This script tests that our HTML content is properly embedded and accessible

echo "🧪 Testing HTML Content..."
echo "================================"

# Test 1: Check if HTML file exists and is readable
if [ -f "src/index.html" ]; then
    echo "✅ HTML file exists: src/index.html"
else
    echo "❌ HTML file missing: src/index.html"
    exit 1
fi

# Test 2: Verify HTML has proper DOCTYPE
if grep -q "<!DOCTYPE html>" src/index.html; then
    echo "✅ HTML5 DOCTYPE declaration found"
else
    echo "❌ HTML5 DOCTYPE declaration missing"
    exit 1
fi

# Test 3: Verify UTF-8 charset meta tag
if grep -q 'charset="UTF-8"' src/index.html; then
    echo "✅ UTF-8 charset meta tag found"
else
    echo "❌ UTF-8 charset meta tag missing"
    exit 1
fi

# Test 4: Verify viewport meta tag
if grep -q 'name="viewport"' src/index.html; then
    echo "✅ Viewport meta tag found"
else
    echo "❌ Viewport meta tag missing"
    exit 1
fi

# Test 5: Check if Rust code compiles successfully
echo ""
echo "🦀 Testing Rust compilation..."
if cargo check --quiet; then
    echo "✅ Rust code compiles successfully"
else
    echo "❌ Rust compilation failed"
    exit 1
fi

# Test 6: Run unit tests
echo ""
echo "🧪 Running unit tests..."
if cargo test --quiet; then
    echo "✅ All unit tests pass"
else
    echo "❌ Unit tests failed"
    exit 1
fi

# Test 7: Verify HTML content is embedded in binary and check actual output
echo ""
echo "🔍 Testing HTML embedding and actual Lambda output..."
echo "Getting actual HTML output from Lambda function..."

# Get the actual HTML output from the Lambda function
HTML_OUTPUT=$(cargo run --bin test_lambda --quiet 2>/dev/null)

if [ $? -eq 0 ]; then
    echo "✅ Lambda function executed successfully"
    
    # Check the actual HTML output for required elements
    echo "Checking actual HTML content structure..."
    
    if echo "$HTML_OUTPUT" | grep -q "<!DOCTYPE html>"; then
        echo "✓ DOCTYPE declaration found in Lambda output"
    else
        echo "❌ DOCTYPE declaration missing in Lambda output"
        exit 1
    fi
    
    if echo "$HTML_OUTPUT" | grep -q 'charset="UTF-8"'; then
        echo "✓ UTF-8 charset meta tag found in Lambda output"
    else
        echo "❌ UTF-8 charset meta tag missing in Lambda output"
        exit 1
    fi
    
    if echo "$HTML_OUTPUT" | grep -q 'name="viewport"'; then
        echo "✓ Viewport meta tag found in Lambda output"
    else
        echo "❌ Viewport meta tag missing in Lambda output"
        exit 1
    fi
    
    if echo "$HTML_OUTPUT" | grep -q "🦀 Rust Lambda Function"; then
        echo "✓ Main heading found in Lambda output"
    else
        echo "❌ Main heading missing in Lambda output"
        exit 1
    fi
    
    echo "✅ All HTML structure checks passed in actual Lambda output"
else
    echo "❌ Lambda function failed to execute"
    exit 1
fi

echo ""
echo "🎉 All tests passed! HTML content is properly configured."
echo "📝 Summary:"
echo "   - HTML file exists and is valid"
echo "   - Required meta tags are present"
echo "   - Rust code compiles without errors"
echo "   - Unit tests pass"
echo "   - HTML content is embedded in the binary"
echo ""
echo "🚀 Ready for deployment!"
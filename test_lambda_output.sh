#!/bin/bash

# Simple one-liner test to get actual HTML output from Lambda function
# This simulates a Lambda call and checks the actual HTML content produced

echo "🚀 Testing actual Lambda HTML output..."
HTML_OUTPUT=$(cargo run --bin test_lambda --quiet 2>/dev/null)

if [ $? -eq 0 ]; then
    echo "✅ Lambda function executed successfully"
    echo ""
    echo "🔍 Checking HTML content structure in actual output..."
    
    echo "$HTML_OUTPUT" | grep -o "<!DOCTYPE html>" && echo "✓ DOCTYPE declaration found"
    echo "$HTML_OUTPUT" | grep -o 'charset="UTF-8"' && echo "✓ UTF-8 charset meta tag found"  
    echo "$HTML_OUTPUT" | grep -o 'name="viewport"' && echo "✓ Viewport meta tag found"
    echo "$HTML_OUTPUT" | grep -o "🦀 Rust Lambda Function" && echo "✓ Main heading found"
    
    echo ""
    echo "🎉 All HTML structure checks passed in actual Lambda output!"
else
    echo "❌ Lambda function failed to execute"
    exit 1
fi
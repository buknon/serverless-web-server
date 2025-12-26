#!/bin/bash

# Task 30: Enhanced Error Logging - COMPREHENSIVE TEST SUITE
# This script runs all tests to validate the Task 30 implementation
# Use this for: Development workflow, CI/CD, automated testing, validation

echo "🧪 TASK 30: Enhanced Error Logging - COMPREHENSIVE TEST SUITE"
echo "=============================================================="
echo "This script validates all aspects of Task 30 implementation:"
echo "- Runs unit tests for enhanced logging functionality"
echo "- Executes integration tests for error response format"  
echo "- Demonstrates live enhanced logging with request IDs"
echo "- Validates Requirements 5.4 compliance"
echo "- Provides pass/fail results for automated testing"
echo "=============================================================="
echo

# Test 1: Run unit tests for enhanced logging
echo "📋 Test 1: Running enhanced logging unit tests..."
RUST_LOG=error cargo test enhanced_logging_tests --lib -- --nocapture
if [ $? -eq 0 ]; then
    echo "✅ Enhanced logging unit tests passed!"
else
    echo "❌ Enhanced logging unit tests failed!"
    exit 1
fi
echo

# Test 2: Run integration test that shows error messages with request IDs
echo "📋 Test 2: Running integration test for error response format..."
RUST_LOG=error cargo test test_error_response_bodies --lib -- --nocapture
if [ $? -eq 0 ]; then
    echo "✅ Error response format test passed!"
else
    echo "❌ Error response format test failed!"
    exit 1
fi
echo

# Test 3: Run the demonstration binary to show enhanced logging in action
echo "📋 Test 3: Running enhanced error logging demonstration..."
echo "This will show the actual log output with request IDs and detailed error information:"
echo
RUST_LOG=error,warn cargo run --bin demo_error_logging
if [ $? -eq 0 ]; then
    echo "✅ Enhanced error logging demonstration completed!"
else
    echo "❌ Enhanced error logging demonstration failed!"
    exit 1
fi
echo

echo "🎉 All Task 30 tests completed successfully!"
echo "========================================================"
echo "✅ Requirements 5.4 Implementation Verified:"
echo "   - Log full error details for debugging: ✅"
echo "   - Keep user responses generic: ✅" 
echo "   - Include request ID for error correlation: ✅"
echo
echo "🔍 Key Features Tested:"
echo "   - Request ID generation (AWS Lambda env vars + fallback)"
echo "   - Detailed internal error logging"
echo "   - Generic user-safe error messages"
echo "   - Structured logging with multiple levels"
echo "   - Error type categorization"
echo "   - Request correlation between logs and user responses"
echo "========================================================"
// Static Web Lambda - A simple Rust-based serverless web server
// 
// This library provides a Lambda function that serves static HTML content
// through AWS Lambda Function URLs. It demonstrates security best practices,
// proper error handling, and comprehensive testing.

// Public modules - these contain the main functionality
pub mod handler;
pub mod response;
pub mod security;

// Test modules - only compiled when running tests
#[cfg(test)]
mod tests;

// Re-export the main handler function for easy access
pub use handler::function_handler;
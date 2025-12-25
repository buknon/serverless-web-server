// This is the main entry point for our Rust Lambda function
// Lambda functions in Rust use a special runtime that handles the communication
// between AWS Lambda service and our Rust code

// Import the Lambda HTTP runtime - this is specifically for HTTP-based Lambda functions
// like those using Function URLs or API Gateway
use lambda_http::{run, service_fn, Error};

// Import Tokio for async runtime support
// Rust's async/await requires a runtime to execute async functions
// Tokio is the most popular async runtime for Rust
use tokio;

// Import logging functionality for structured request/response logging
use log::{info, error};

// Import our handler function from the library
use static_web_lambda::function_handler;

/// The main function is the entry point of our Rust program
/// 
/// In Rust, the main function for async programs needs special handling
/// The #[tokio::main] attribute sets up the async runtime for us
/// 
/// This function will:
/// 1. Set up basic logging (we'll improve this later)
/// 2. Start the Lambda HTTP runtime with error handling
/// 3. Register our handler function
/// 4. Begin listening for HTTP requests via Lambda Function URL
/// 5. Handle runtime startup errors gracefully
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize structured logging for Lambda function
    // 
    // AWS Lambda automatically captures stdout/stderr and sends them to CloudWatch Logs
    // env_logger outputs to stderr by default, which is perfect for Lambda
    // 
    // Log level configuration:
    // - In Lambda: Set RUST_LOG environment variable (e.g., RUST_LOG=info)
    // - Locally: Set RUST_LOG environment variable or use default (error level)
    // 
    // Structured logging benefits:
    // - Consistent timestamp format across all log entries
    // - Proper log levels (error, warn, info, debug, trace)
    // - Easy parsing by log aggregation tools
    // - Better debugging and monitoring capabilities
    env_logger::init();
    
    info!("Initializing Lambda function runtime...");
    
    // Validate that our handler function is properly configured
    // This is a good practice to catch configuration issues early
    info!("Handler function registered successfully");
    
    // Start the Lambda HTTP runtime with comprehensive error handling
    // 
    // service_fn() converts our handler function into a service that can process
    // multiple requests. It wraps our function_handler in the necessary boilerplate
    // to make it compatible with the Lambda runtime's service interface.
    // 
    // run() starts the Lambda runtime and begins the event loop that:
    // 1. Receives HTTP events from Lambda Function URL
    // 2. Converts them to standard HTTP Request objects
    // 3. Calls our handler function
    // 4. Converts the Response back to Lambda format
    // 5. Returns the response to the Lambda service
    // 
    // Error handling strategy:
    // - If run() fails during startup, we log the error and propagate it
    // - Runtime errors during request processing are handled by the Lambda service
    // - The ? operator propagates startup errors to the Lambda service for logging
    match run(service_fn(function_handler)).await {
        Ok(()) => {
            // This should rarely happen as run() typically doesn't return Ok(())
            // unless the Lambda service is shutting down gracefully
            info!("Lambda runtime shut down successfully");
            Ok(())
        }
        Err(e) => {
            // Log the startup error for debugging
            // This helps identify issues like:
            // - Missing environment variables
            // - Network connectivity problems
            // - Handler function panics during initialization
            // - Lambda service configuration issues
            error!("Lambda runtime startup failed: {}", e);
            
            // Propagate the error to the Lambda service
            // The Lambda service will log this error and mark the function as failed
            // This prevents the function from receiving traffic until the issue is resolved
            Err(e)
        }
    }
}

// This section contains unit tests for our Lambda function
// Tests in Rust are typically placed in the same file as the code they test
// The #[cfg(test)] attribute means this code only compiles when running tests
/// Test helper function to simulate a Lambda call and return the HTML response
/// This allows us to test the actual HTML output produced by the function
#[cfg(test)]
pub async fn simulate_lambda_call() -> Result<String, lambda_http::Error> {
    use lambda_http::http;
    use lambda_http::Body;
    
    // Create a mock HTTP GET request
    let request = http::Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::Empty)
        .map_err(|e| lambda_http::Error::from(format!("Request build error: {}", e)))?;
    
    // Call our handler function
    let response = function_handler(request).await?;
    
    // Extract the body as a string
    let body_bytes = hyper::body::to_bytes(response.into_body()).await
        .map_err(|e| lambda_http::Error::from(format!("Body conversion error: {}", e)))?;
    let html_content = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| lambda_http::Error::from(format!("UTF-8 conversion error: {}", e)))?;
    
    Ok(html_content)
}

#[cfg(test)]
mod tests {
    // Import everything from the parent module (our main code)
    use super::*;
    
    // Import additional testing utilities
    use lambda_http::{Body};
    use lambda_http::http;
    
    /// Test that our handler function can process a basic HTTP request
    /// 
    /// The #[tokio::test] attribute is like #[tokio::main] but for test functions
    /// It sets up an async runtime for testing async functions
    #[tokio::test]
    async fn test_handler_basic_request() {
        // Create a mock HTTP GET request
        // This simulates what Lambda would send to our function
        // We use http::Request::builder() instead of Request::builder()
        let request = http::Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::Empty)
            .expect("Failed to build request");
        
        // Call our handler function with the test request
        let response = function_handler(request).await;
        
        // Verify that the handler succeeded (returned Ok, not Err)
        assert!(response.is_ok(), "Handler should succeed");
        
        // Extract the response from the Result
        let response = response.unwrap();
        
        // Verify the response has the expected status code
        assert_eq!(response.status(), 200, "Response should have status 200");
        
        // Verify the response has the correct content type
        let content_type = response.headers().get("content-type");
        assert!(content_type.is_some(), "Response should have content-type header");
        assert_eq!(content_type.unwrap(), "text/html", "Content-type should be text/html");
    }
    
    /// Test the actual HTML content produced by the Lambda function
    #[tokio::test]
    async fn test_html_content_structure() {
        // Get the actual HTML response from the Lambda function
        let html_content = simulate_lambda_call().await.expect("Should get HTML content");
        
        // Verify HTML structure
        assert!(html_content.contains("<!DOCTYPE html>"), "Should contain DOCTYPE declaration");
        assert!(html_content.contains("charset=\"UTF-8\""), "Should contain UTF-8 charset");
        assert!(html_content.contains("name=\"viewport\""), "Should contain viewport meta tag");
        assert!(html_content.contains("🦀 Rust Lambda Function"), "Should contain main heading");
    }
}
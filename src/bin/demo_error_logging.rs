// Demonstration of enhanced error logging with request IDs
// This binary shows the enhanced error logging implemented in Task 30

use static_web_lambda::function_handler;
use lambda_http::{Body, http, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging to show error messages
    std::env::set_var("RUST_LOG", "error,warn");
    env_logger::init();
    
    let separator = "=".repeat(80);
    
    println!("{}", separator);
    println!("ENHANCED ERROR LOGGING DEMONSTRATION");
    println!("{}", separator);
    println!("This demonstrates the enhanced error logging implemented in Task 30.");
    println!("Each error now includes:");
    println!("- Unique Request ID for correlation");
    println!("- Detailed error information in logs");
    println!("- Generic user-safe messages in responses");
    println!("- Structured logging for different error types");
    println!("{}", separator);
    println!();
    
    // Test 1: Invalid HTTP method (should trigger security error)
    println!("=== Test 1: Invalid HTTP Method (POST) ===");
    println!("Expected: 405 Method Not Allowed with detailed security logging");
    println!();
    
    let request = http::Request::builder()
        .method("POST")
        .uri("/")
        .body(Body::Empty)
        .map_err(|e| Error::from(e.to_string()))?;
    
    let response = function_handler(request).await?;
    println!("Response Status: {}", response.status());
    
    let body_bytes = hyper::body::to_bytes(response.into_body()).await
        .map_err(|e| Error::from(e.to_string()))?;
    let body_content = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| Error::from(e.to_string()))?;
    println!("User Response: {}", body_content);
    println!();
    
    // Test 2: Malicious path (should trigger security error)
    println!("=== Test 2: Directory Traversal Attack ===");
    println!("Expected: 400 Bad Request with detailed security logging");
    println!();
    
    let request = http::Request::builder()
        .method("GET")
        .uri("/../etc/passwd")
        .body(Body::Empty)
        .map_err(|e| Error::from(e.to_string()))?;
    
    let response = function_handler(request).await?;
    println!("Response Status: {}", response.status());
    
    let body_bytes = hyper::body::to_bytes(response.into_body()).await
        .map_err(|e| Error::from(e.to_string()))?;
    let body_content = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| Error::from(e.to_string()))?;
    println!("User Response: {}", body_content);
    println!();
    
    println!("{}", separator);
    println!("DEMONSTRATION COMPLETE");
    println!("{}", separator);
    println!("Key observations:");
    println!("1. Each error has a unique Request ID for correlation");
    println!("2. Detailed logs show full error context for debugging");
    println!("3. User responses are generic and don't leak sensitive info");
    println!("4. Different error types have structured logging categories");
    println!("5. Request IDs enable linking user reports to internal logs");
    
    Ok(())
}
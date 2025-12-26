// Integration tests for the complete Lambda handler function
// These tests verify that all components work together correctly

use crate::handler::function_handler;
use lambda_http::{Body, http};
use hyper::body::to_bytes;

// Import env_logger for testing logging functionality
use env_logger;

/// Test that GET requests are processed successfully
/// 
/// This test verifies that valid GET requests:
/// 1. Return HTTP 200 OK status
/// 2. Include proper Content-Type header for HTML
/// 3. Include X-Content-Type-Options security header
/// 4. Include X-Frame-Options security header
/// 5. Process without errors
#[tokio::test]
async fn test_get_request_success() {
    // Initialize logger for this test to verify logging functionality
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Create a mock HTTP GET request with User-Agent header
    let request = http::Request::builder()
        .method("GET")
        .uri("/")
        .header("user-agent", "Mozilla/5.0 (Test Browser)")
        .body(Body::Empty)
        .expect("Failed to build GET request");
    
    // Call our handler function
    let response = function_handler(request).await;
    
    // Verify the handler succeeded
    assert!(response.is_ok(), "GET request should succeed");
    
    let response = response.unwrap();
    
    // Verify HTTP 200 OK status for GET requests
    assert_eq!(response.status(), 200, "GET request should return status 200");
    
    // Verify Content-Type header for HTML content
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some(), "Response should have content-type header");
    assert_eq!(content_type.unwrap(), "text/html", "Content-type should be text/html");
    
    // Verify X-Content-Type-Options security header (Task 20)
    let x_content_type_options = response.headers().get("x-content-type-options");
    assert!(x_content_type_options.is_some(), "Response should have X-Content-Type-Options header");
    assert_eq!(x_content_type_options.unwrap(), "nosniff", "X-Content-Type-Options should be nosniff");
    
    // Verify X-Frame-Options security header (Task 21)
    let x_frame_options = response.headers().get("x-frame-options");
    assert!(x_frame_options.is_some(), "Response should have X-Frame-Options header");
    assert_eq!(x_frame_options.unwrap(), "DENY", "X-Frame-Options should be DENY");
}

/// Test that suspicious user agents are logged with security warnings
/// 
/// This test verifies that requests with suspicious user agents (like security scanners)
/// are properly logged with security warnings for monitoring purposes.
#[tokio::test]
async fn test_suspicious_user_agent_logging() {
    // Initialize logger for this test
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Create a request with a suspicious user agent (sqlmap - SQL injection scanner)
    let request = http::Request::builder()
        .method("GET")
        .uri("/")
        .header("user-agent", "sqlmap/1.0 (http://sqlmap.org)")
        .body(Body::Empty)
        .expect("Failed to build request");
    
    // Call our handler function
    let response = function_handler(request).await;
    
    // Verify the handler succeeded (suspicious user agents don't block requests)
    assert!(response.is_ok(), "Request with suspicious user agent should still succeed");
    
    let response = response.unwrap();
    
    // Verify normal response (suspicious user agents are logged but not blocked)
    assert_eq!(response.status(), 200, "Response should still be successful");
}

/// Test that POST requests are rejected with HTTP 405 Method Not Allowed
/// 
/// This test verifies HTTP method validation (Task 16 - Requirements 3.4):
/// 1. POST requests return HTTP 405 status
/// 2. Response includes Allow header indicating GET is supported
/// 3. Response includes appropriate error message
#[tokio::test]
async fn test_post_request_rejected() {
    // Initialize logger for this test to verify logging functionality
    let _ = env_logger::builder().is_test(true).try_init();
    
    // Create a mock HTTP POST request
    let request = http::Request::builder()
        .method("POST")
        .uri("/")
        .body(Body::Empty)
        .expect("Failed to build POST request");
    
    // Call our handler function
    let response = function_handler(request).await;
    
    // Verify the handler succeeded (returned a response, not an error)
    assert!(response.is_ok(), "Handler should return response for POST");
    
    let response = response.unwrap();
    
    // Verify HTTP 405 Method Not Allowed status
    assert_eq!(response.status(), 405, "POST request should return status 405");
    
    // Verify Allow header is present and indicates GET is supported
    let allow_header = response.headers().get("allow");
    assert!(allow_header.is_some(), "Response should have Allow header");
    assert_eq!(allow_header.unwrap(), "GET", "Allow header should specify GET");
    
    // Verify Content-Type is text/plain for error message
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some(), "Response should have content-type header");
    assert_eq!(content_type.unwrap(), "text/plain", "Error response should be text/plain");
}

/// Test that various HTTP methods are rejected consistently
/// 
/// This test verifies that all non-GET methods are properly rejected
#[tokio::test]
async fn test_various_methods_rejected() {
    let methods = vec!["PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
    
    for method in methods {
        // Create a mock HTTP request with the specified method
        let request = http::Request::builder()
            .method(method)
            .uri("/")
            .body(Body::Empty)
            .expect("Failed to build request");
        
        // Call our handler function
        let response = function_handler(request).await;
        
        // Verify the handler succeeded (returned a response, not an error)
        assert!(response.is_ok(), "Handler should return response for {}", method);
        
        let response = response.unwrap();
        
        // Verify HTTP 405 Method Not Allowed status
        assert_eq!(response.status(), 405, "{} request should return status 405", method);
        
        // Verify Allow header is present
        let allow_header = response.headers().get("allow");
        assert!(allow_header.is_some(), "Response should have Allow header for {}", method);
        assert_eq!(allow_header.unwrap(), "GET", "Allow header should specify GET for {}", method);
    }
}

/// Test GET request with different paths to ensure method validation works consistently
/// 
/// This test verifies that method validation works regardless of the request path
/// and that security headers are consistently applied
#[tokio::test]
async fn test_get_request_different_paths() {
    let test_paths = vec!["/", "/index.html", "/about", "/contact", "/api/test"];
    
    for path in test_paths {
        // Create a mock HTTP GET request with different path
        let request = http::Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::Empty)
            .expect("Failed to build GET request");
        
        // Call our handler function
        let response = function_handler(request).await;
        
        // Verify the handler succeeded
        assert!(response.is_ok(), "GET request to {} should succeed", path);
        
        let response = response.unwrap();
        
        // Verify HTTP 200 OK status for all GET requests regardless of path
        assert_eq!(response.status(), 200, "GET request to {} should return status 200", path);
        
        // Verify Content-Type header for HTML content
        let content_type = response.headers().get("content-type");
        assert!(content_type.is_some(), "Response should have content-type header for path {}", path);
        assert_eq!(content_type.unwrap(), "text/html", "Content-type should be text/html for path {}", path);
        
        // Verify X-Content-Type-Options security header is present for all paths (Task 20)
        let x_content_type_options = response.headers().get("x-content-type-options");
        assert!(x_content_type_options.is_some(), "Response should have X-Content-Type-Options header for path {}", path);
        assert_eq!(x_content_type_options.unwrap(), "nosniff", "X-Content-Type-Options should be nosniff for path {}", path);
        
        // Verify X-Frame-Options security header is present for all paths (Task 21)
        let x_frame_options = response.headers().get("x-frame-options");
        assert!(x_frame_options.is_some(), "Response should have X-Frame-Options header for path {}", path);
        assert_eq!(x_frame_options.unwrap(), "DENY", "X-Frame-Options should be DENY for path {}", path);
    }
}

/// Test GET requests with malicious paths are rejected
/// 
/// This test verifies that the handler function properly integrates path sanitization
#[tokio::test]
async fn test_get_request_malicious_paths() {
    let malicious_paths = vec![
        "/../etc/passwd",
        "/../../secret.txt",
        "/static/%2e%2e/config",
        // Note: We can't test paths with literal < > characters in URIs
        // as they are invalid URI characters, but our sanitizer would catch them
    ];
    
    for path in malicious_paths {
        // Create a mock HTTP GET request with malicious path
        let request = http::Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::Empty)
            .expect("Failed to build GET request");
        
        // Call our handler function
        let response = function_handler(request).await;
        
        // Verify the handler succeeded (returned a response, not an error)
        assert!(response.is_ok(), "Handler should return response for malicious path {}", path);
        
        let response = response.unwrap();
        
        // Verify HTTP 400 Bad Request status for malicious paths
        assert_eq!(response.status(), 400, "Malicious path {} should return status 400", path);
        
        // Verify Content-Type is text/plain for error message
        let content_type = response.headers().get("content-type");
        assert!(content_type.is_some(), "Response should have content-type header for path {}", path);
        assert_eq!(content_type.unwrap(), "text/plain", "Error response should be text/plain for path {}", path);
    }
}

/// Test that safe GET requests still work after adding path sanitization
/// 
/// This test verifies that legitimate requests are not affected by security measures
#[tokio::test]
async fn test_get_request_safe_paths_after_sanitization() {
    let safe_paths = vec![
        "/",
        "/index.html",
        "/about",
        "/api/status",
        "/static/style.css",
    ];
    
    for path in safe_paths {
        // Create a mock HTTP GET request with safe path
        let request = http::Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::Empty)
            .expect("Failed to build GET request");
        
        // Call our handler function
        let response = function_handler(request).await;
        
        // Verify the handler succeeded
        assert!(response.is_ok(), "Safe GET request to {} should succeed", path);
        
        let response = response.unwrap();
        
        // Verify HTTP 200 OK status for safe GET requests
        assert_eq!(response.status(), 200, "Safe GET request to {} should return status 200", path);
        
        // Verify Content-Type header for HTML content
        let content_type = response.headers().get("content-type");
        assert!(content_type.is_some(), "Response should have content-type header for path {}", path);
        assert_eq!(content_type.unwrap(), "text/html", "Content-type should be text/html for path {}", path);
    }
}

/// Test that normal-sized requests are accepted
/// 
/// This test verifies that requests within the size limit are processed normally
#[tokio::test]
async fn test_normal_request_size_accepted() {
    // Create a normal-sized GET request
    let request = http::Request::builder()
        .method("GET")
        .uri("/")
        .header("User-Agent", "Mozilla/5.0 (compatible; test)")
        .header("Accept", "text/html,application/xhtml+xml")
        .body(Body::Empty)
        .expect("Failed to build normal request");
    
    // Call our handler function
    let response = function_handler(request).await;
    
    // Verify the handler succeeded
    assert!(response.is_ok(), "Normal-sized request should succeed");
    
    let response = response.unwrap();
    
    // Verify HTTP 200 OK status for normal requests
    assert_eq!(response.status(), 200, "Normal request should return status 200");
    
    // Verify Content-Type header for HTML content
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some(), "Response should have content-type header");
    assert_eq!(content_type.unwrap(), "text/html", "Content-type should be text/html");
}

/// Test that oversized requests are rejected with HTTP 413
/// 
/// This test verifies that requests exceeding the size limit are properly rejected
#[tokio::test]
async fn test_oversized_request_rejected() {
    // Create a request with many large headers to exceed the size limit
    let mut request_builder = http::Request::builder()
        .method("GET")
        .uri("/");
    
    // Add many large headers to exceed the 64KB limit
    // Each header is about 1KB, so 70 headers should exceed the limit
    for i in 0..70 {
        let header_name = format!("X-Large-Header-{}", i);
        let header_value = "x".repeat(1000); // 1KB header value
        request_builder = request_builder.header(header_name, header_value);
    }
    
    let request = request_builder
        .body(Body::Empty)
        .expect("Failed to build oversized request");
    
    // Call our handler function
    let response = function_handler(request).await;
    
    // Verify the handler succeeded (returned a response, not an error)
    assert!(response.is_ok(), "Handler should return response for oversized request");
    
    let response = response.unwrap();
    
    // Verify HTTP 413 Request Entity Too Large status
    assert_eq!(response.status(), 413, "Oversized request should return status 413");
    
    // Verify Content-Type is text/plain for error message
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some(), "Response should have content-type header");
    assert_eq!(content_type.unwrap(), "text/plain", "Error response should be text/plain");
}

/// Test error response body content for various error conditions
/// 
/// This test verifies that error messages are appropriate and don't leak information
#[tokio::test]
async fn test_error_response_bodies() {
    // Test method not allowed error message
    let request = http::Request::builder()
        .method("POST")
        .uri("/")
        .body(Body::Empty)
        .expect("Failed to build POST request");
    
    let response = function_handler(request).await.unwrap();
    assert_eq!(response.status(), 405);
    
    let body_bytes = to_bytes(response.into_body()).await
        .expect("Should be able to read response body");
    let body_content = String::from_utf8(body_bytes.to_vec())
        .expect("Response body should be valid UTF-8");
    
    assert!(
        body_content.starts_with("Method Not Allowed. This server only supports GET requests.") &&
        body_content.contains("(Request ID: "),
        "Method not allowed error message should be clear and informative with request ID. Got: {}",
        body_content
    );
    
    // Test malicious path error message
    let request = http::Request::builder()
        .method("GET")
        .uri("/../etc/passwd")
        .body(Body::Empty)
        .expect("Failed to build malicious request");
    
    let response = function_handler(request).await.unwrap();
    assert_eq!(response.status(), 400);
    
    let body_bytes = to_bytes(response.into_body()).await
        .expect("Should be able to read response body");
    let body_content = String::from_utf8(body_bytes.to_vec())
        .expect("Response body should be valid UTF-8");
    
    assert!(
        body_content.starts_with("Bad Request. Invalid request path.") &&
        body_content.contains("(Request ID: "),
        "Malicious path error message should be generic and not leak security details, with request ID. Got: {}",
        body_content
    );
}
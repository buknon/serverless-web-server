// Unit tests for individual functions and components
// These tests focus on testing specific functions in isolation

use crate::response::{create_html_response, create_error_response};
use crate::security::{sanitize_path, validate_http_method};

/// Test the create_html_response function directly
/// 
/// This test verifies that our HTML response creation function:
/// 1. Returns a successful Result (no errors)
/// 2. Sets the correct HTTP status code (200)
/// 3. Sets the correct Content-Type header (text/html)
/// 4. Sets the X-Content-Type-Options security header (nosniff)
/// 5. Sets the X-Frame-Options security header (DENY)
/// 6. Includes the expected HTML content in the body
#[test]
fn test_create_html_response() {
    // Call our HTML response creation function
    let result = create_html_response();
    
    // Verify the function succeeded
    assert!(result.is_ok(), "create_html_response should succeed");
    
    let response = result.unwrap();
    
    // Verify HTTP status code is 200 (OK)
    assert_eq!(response.status(), 200, "Response should have status 200");
    
    // Verify Content-Type header is set correctly
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some(), "Response should have content-type header");
    assert_eq!(content_type.unwrap(), "text/html", "Content-type should be text/html");
    
    // Verify X-Content-Type-Options security header is set correctly (Task 20)
    let x_content_type_options = response.headers().get("x-content-type-options");
    assert!(x_content_type_options.is_some(), "Response should have X-Content-Type-Options header");
    assert_eq!(x_content_type_options.unwrap(), "nosniff", "X-Content-Type-Options should be nosniff");
    
    // Verify X-Frame-Options security header is set correctly (Task 21)
    let x_frame_options = response.headers().get("x-frame-options");
    assert!(x_frame_options.is_some(), "Response should have X-Frame-Options header");
    assert_eq!(x_frame_options.unwrap(), "DENY", "X-Frame-Options should be DENY");
    
    // Verify Content-Security-Policy security header is set correctly (Task 22)
    let csp = response.headers().get("content-security-policy");
    assert!(csp.is_some(), "Response should have Content-Security-Policy header");
    let expected_csp = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";
    assert_eq!(csp.unwrap(), expected_csp, "Content-Security-Policy should restrict resource loading");
    
    // Verify the response body contains our HTML content
    // Note: We can't easily test the body content here because Response<Body>
    // doesn't provide direct access to the body in tests. The integration
    // tests handle testing the actual HTML content.
}

/// Test the create_error_response function
/// 
/// This test verifies that error responses are created correctly
#[test]
fn test_create_error_response() {
    let result = create_error_response(400, "Bad Request");
    
    assert!(result.is_ok(), "create_error_response should succeed");
    
    let response = result.unwrap();
    
    // Verify status code
    assert_eq!(response.status(), 400, "Response should have status 400");
    
    // Verify Content-Type header for error responses
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some(), "Response should have content-type header");
    assert_eq!(content_type.unwrap(), "text/plain", "Error response should be text/plain");
    
    // Verify security header is present on error responses too
    let x_content_type_options = response.headers().get("x-content-type-options");
    assert!(x_content_type_options.is_some(), "Error response should have X-Content-Type-Options header");
    assert_eq!(x_content_type_options.unwrap(), "nosniff", "X-Content-Type-Options should be nosniff");
    
    // Verify X-Frame-Options security header is present on error responses (Task 21)
    let x_frame_options = response.headers().get("x-frame-options");
    assert!(x_frame_options.is_some(), "Error response should have X-Frame-Options header");
    assert_eq!(x_frame_options.unwrap(), "DENY", "X-Frame-Options should be DENY");
    
    // Verify Content-Security-Policy security header is present on error responses (Task 22)
    let csp = response.headers().get("content-security-policy");
    assert!(csp.is_some(), "Error response should have Content-Security-Policy header");
    let expected_csp = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";
    assert_eq!(csp.unwrap(), expected_csp, "Content-Security-Policy should restrict resource loading");
}

/// Test the sanitize_path function directly with safe paths
/// 
/// This test verifies that legitimate paths are accepted by the sanitization function
#[test]
fn test_sanitize_path_safe_paths() {
    // Test various safe paths that should be accepted
    let safe_paths = vec![
        "/",
        "/index.html",
        "/about",
        "/contact.html",
        "/api/v1/status",
        "/static/css/style.css",
        "/images/logo.png",
        "/docs/readme.txt",
    ];
    
    for path in safe_paths {
        let result = sanitize_path(path);
        assert!(result.is_ok(), "Safe path '{}' should be accepted", path);
        assert_eq!(result.unwrap(), path, "Safe path '{}' should be returned unchanged", path);
    }
}

/// Test the sanitize_path function with directory traversal attacks
/// 
/// This test verifies that various directory traversal patterns are rejected
#[test]
fn test_sanitize_path_directory_traversal() {
    // Test various directory traversal attack patterns
    let malicious_paths = vec![
        "../",
        "../../",
        "../../../etc/passwd",
        "/../../etc/passwd",
        "/static/../../../secret.txt",
        "/../etc/hosts",
        "/./../../etc/passwd",
        "/normal/../../../etc/passwd",
    ];
    
    for path in malicious_paths {
        let result = sanitize_path(path);
        assert!(result.is_err(), "Malicious path '{}' should be rejected", path);
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("parent directory") || error_msg.contains("current directory"),
            "Error message for '{}' should mention directory reference: {}",
            path,
            error_msg
        );
    }
}

/// Test the sanitize_path function with encoded traversal attacks
/// 
/// This test verifies that URL-encoded directory traversal patterns are rejected
#[test]
fn test_sanitize_path_encoded_traversal() {
    // Test various encoded directory traversal attack patterns
    let encoded_attacks = vec![
        "/static/%2e%2e/secret.txt",
        "/api/%2E%2E/admin",
        "/files/%2e%2E/config",
        "/docs/..%2f../etc/passwd",
        "/images/.%2e/sensitive",
    ];
    
    for path in encoded_attacks {
        let result = sanitize_path(path);
        assert!(result.is_err(), "Encoded attack path '{}' should be rejected", path);
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("encoded traversal"),
            "Error message for '{}' should mention encoded traversal: {}",
            path,
            error_msg
        );
    }
}

/// Test the sanitize_path function with dangerous characters
/// 
/// This test verifies that paths containing dangerous characters are rejected
#[test]
fn test_sanitize_path_dangerous_characters() {
    // Test paths with dangerous characters that could be used in injection attacks
    let dangerous_paths = vec![
        "/path<script>alert('xss')</script>",
        "/file\"with\"quotes",
        "/path'with'quotes",
        "/file&with&ampersands",
        "/path\nwith\nnewlines",
        "/path\rwith\rcarriage\rreturns",
        "/path\twith\ttabs",
    ];
    
    for path in dangerous_paths {
        let result = sanitize_path(path);
        assert!(result.is_err(), "Path with dangerous characters '{}' should be rejected", path);
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("dangerous character"),
            "Error message for '{}' should mention dangerous character: {}",
            path,
            error_msg
        );
    }
}

/// Test the sanitize_path function with null byte injection
/// 
/// This test verifies that paths containing null bytes are rejected
#[test]
fn test_sanitize_path_null_bytes() {
    // Test paths with null bytes that could be used for path truncation attacks
    let null_byte_paths = vec![
        "/safe/path\0../../etc/passwd",
        "/file\0.txt",
        "\0/etc/passwd",
    ];
    
    for path in null_byte_paths {
        let result = sanitize_path(path);
        assert!(result.is_err(), "Path with null byte should be rejected");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("null byte"),
            "Error message should mention null byte: {}",
            error_msg
        );
    }
}

/// Test the sanitize_path function with excessively long paths
/// 
/// This test verifies that very long paths are rejected to prevent DoS attacks
#[test]
fn test_sanitize_path_long_paths() {
    // Create a path that exceeds the maximum allowed length
    let long_path = "/".to_string() + &"a".repeat(1001);
    
    let result = sanitize_path(&long_path);
    assert!(result.is_err(), "Excessively long path should be rejected");
    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("too long"),
        "Error message should mention path length: {}",
        error_msg
    );
}

/// Test the validate_http_method function
/// 
/// This test verifies that only GET methods are allowed
#[test]
fn test_validate_http_method() {
    // GET should be allowed
    assert!(validate_http_method("GET").is_ok(), "GET method should be allowed");
    
    // Other methods should be rejected
    let invalid_methods = vec!["POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
    
    for method in invalid_methods {
        let result = validate_http_method(method);
        assert!(result.is_err(), "Method '{}' should be rejected", method);
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("not allowed"),
            "Error message for '{}' should mention not allowed: {}",
            method,
            error_msg
        );
    }
}
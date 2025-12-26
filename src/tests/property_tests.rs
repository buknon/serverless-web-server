// Property-based tests using proptest
// These tests validate universal properties across many generated inputs

use crate::handler::function_handler;
use crate::security::sanitize_path;
use proptest::prelude::*;
use lambda_http::{Body, http};

// Property-based test for input sanitization
// 
// **Property 8: Input Sanitization**
// **Validates: Requirements 3.4**
// 
// This property test verifies that the input sanitization function correctly
// identifies and rejects malicious inputs while accepting safe inputs.
// 
// The property being tested is:
// "For any HTTP request, the system should sanitize and validate all input 
// parameters to prevent injection attacks"
// 
// This test generates various types of potentially malicious paths and verifies
// that our sanitization function correctly categorizes them as safe or unsafe.
// 
// Feature: static-web-lambda, Property 8: Input Sanitization
proptest! {
    #[test]
    fn test_input_sanitization_property(
        // Generate safe path components
        safe_segments in prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 0..5),
        // Generate potentially malicious components
        has_traversal in any::<bool>(),
        has_dangerous_chars in any::<bool>(),
        has_encoded_traversal in any::<bool>(),
        has_null_bytes in any::<bool>(),
        is_too_long in any::<bool>(),
    ) {
        // Build a test path based on the generated parameters
        let mut test_path = String::from("/");
        
        // Add safe segments first
        if !safe_segments.is_empty() {
            test_path.push_str(&safe_segments.join("/"));
        }
        
        // Add malicious components based on flags
        let mut is_malicious = false;
        
        if has_traversal {
            test_path.push_str("/../secret");
            is_malicious = true;
        }
        
        if has_dangerous_chars {
            test_path.push_str("/<script>");
            is_malicious = true;
        }
        
        if has_encoded_traversal {
            test_path.push_str("/%2e%2e/admin");
            is_malicious = true;
        }
        
        if has_null_bytes {
            test_path.push('\0');
            is_malicious = true;
        }
        
        if is_too_long {
            test_path.push_str(&"a".repeat(2000));
            is_malicious = true;
        }
        
        // Test the sanitize_path function directly
        let sanitization_result = sanitize_path(&test_path);
        
        if is_malicious {
            // Malicious paths should be rejected
            prop_assert!(
                sanitization_result.is_err(),
                "Malicious path '{}' should be rejected by sanitization", 
                test_path
            );
            
            // Test that malicious paths are rejected by the full handler
            // Only test if the path is not too long for HTTP URI parsing
            if test_path.len() < 1000 {
                let request_result = http::Request::builder()
                    .method("GET")
                    .uri(&test_path)
                    .body(Body::Empty);
                
                if let Ok(request) = request_result {
                    let response = tokio_test::block_on(function_handler(request));
                    prop_assert!(
                        response.is_ok(),
                        "Handler should return response (not error) for malicious path '{}'",
                        test_path
                    );
                    
                    let response = response.unwrap();
                    // Malicious paths should return 400 Bad Request or 413 if too large
                    prop_assert!(
                        response.status() == 400 || response.status() == 413,
                        "Malicious path '{}' should return 400 or 413, got {}",
                        test_path,
                        response.status()
                    );
                }
            }
        } else {
            // Safe paths should be accepted
            prop_assert!(
                sanitization_result.is_ok(),
                "Safe path '{}' should be accepted by sanitization", 
                test_path
            );
            
            // Test that safe paths work in the full handler
            if test_path.len() < 1000 { // Avoid excessively long URIs
                let request_result = http::Request::builder()
                    .method("GET")
                    .uri(&test_path)
                    .body(Body::Empty);
                
                if let Ok(request) = request_result {
                    let response = tokio_test::block_on(function_handler(request));
                    prop_assert!(
                        response.is_ok(),
                        "Safe path '{}' should not cause handler errors",
                        test_path
                    );
                    
                    let response = response.unwrap();
                    // Safe paths should return 200 (success)
                    prop_assert_eq!(
                        response.status(),
                        200,
                        "Safe path '{}' should return 200 OK",
                        test_path
                    );
                }
            }
        }
    }
}

// Property test for specific directory traversal patterns
// 
// This test focuses specifically on directory traversal attack patterns
// to ensure comprehensive coverage of this attack vector.
proptest! {
    #[test]
    fn test_directory_traversal_patterns_property(
        // Generate different numbers of "../" sequences
        traversal_count in 1usize..10,
        // Generate different target files
        target_file in "[a-z]{1,20}",
        // Generate different prefixes
        prefix in prop::option::of("[a-zA-Z0-9/]{0,20}"),
    ) {
        // Build directory traversal path
        let traversal_sequence = "../".repeat(traversal_count);
        let test_path = match prefix {
            Some(p) => format!("/{}/{}{}", p, traversal_sequence, target_file),
            None => format!("/{}{}", traversal_sequence, target_file),
        };
        
        // All directory traversal patterns should be rejected
        let sanitization_result = sanitize_path(&test_path);
        prop_assert!(
            sanitization_result.is_err(),
            "Directory traversal path '{}' should be rejected",
            test_path
        );
        
        // Test with the full handler
        if test_path.len() < 1000 {
            let request_result = http::Request::builder()
                .method("GET")
                .uri(&test_path)
                .body(Body::Empty);
            
            if let Ok(request) = request_result {
                let response = tokio_test::block_on(function_handler(request));
                prop_assert!(
                    response.is_ok(),
                    "Handler should return response for traversal path '{}'",
                    test_path
                );
                
                let response = response.unwrap();
                prop_assert_eq!(
                    response.status(),
                    400,
                    "Directory traversal path '{}' should return 400 Bad Request",
                    test_path
                );
            }
        }
    }
}

// Property test for encoded traversal patterns
// 
// This test focuses on URL-encoded directory traversal patterns
// to ensure our sanitization catches encoded attacks.
proptest! {
    #[test]
    fn test_encoded_traversal_patterns_property(
        // Generate different encoded patterns
        encoding_pattern in prop::sample::select(vec![
            "%2e%2e", "%2E%2E", "%2e%2E", "%2E%2e",
            "..%2f", "..%2F", "%2e.", ".%2e"
        ]),
        // Generate path context
        path_context in "[a-zA-Z0-9/]{0,20}",
        target in "[a-z]{1,10}",
    ) {
        let test_path = format!("/{}/{}/{}", path_context, encoding_pattern, target);
        
        // All encoded traversal patterns should be rejected
        let sanitization_result = sanitize_path(&test_path);
        prop_assert!(
            sanitization_result.is_err(),
            "Encoded traversal path '{}' should be rejected",
            test_path
        );
        
        // Test with the full handler
        let request_result = http::Request::builder()
            .method("GET")
            .uri(&test_path)
            .body(Body::Empty);
        
        if let Ok(request) = request_result {
            let response = tokio_test::block_on(function_handler(request));
            prop_assert!(
                response.is_ok(),
                "Handler should return response for encoded traversal path '{}'",
                test_path
            );
            
            let response = response.unwrap();
            prop_assert_eq!(
                response.status(),
                400,
                "Encoded traversal path '{}' should return 400 Bad Request",
                test_path
            );
        }
    }
}

// Property test for security headers validation
// 
// **Property 7: Security Header Validation**
// **Validates: Requirements 3.4**
// 
// This property test verifies that all HTTP responses include the required
// security headers to protect against various web vulnerabilities.
// 
// The property being tested is:
// "For any HTTP response, the system should include appropriate security headers
// (X-Content-Type-Options, X-Frame-Options, Content-Security-Policy)"
// 
// This test generates various types of HTTP requests and verifies that all
// responses (both successful and error responses) include the complete set
// of security headers with correct values.
// 
// Feature: static-web-lambda, Property 7: Security Header Validation
proptest! {
    #[test]
    fn test_security_headers_property(
        // Generate different request paths (safe and unsafe)
        path in "[/a-zA-Z0-9._-]{1,50}",
        // Generate different HTTP methods
        method in prop::sample::select(vec!["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS"]),
        // Generate different request scenarios
        add_malicious_path in any::<bool>(),
        make_oversized_request in any::<bool>(),
    ) {
        // Build test request based on generated parameters
        let mut test_path = path;
        
        // Add malicious content to some paths to test error responses
        if add_malicious_path {
            test_path.push_str("/../secret");
        }
        
        // Create the HTTP request
        let request_result = http::Request::builder()
            .method(&method[..])
            .uri(&test_path);
        
        // Add oversized body for some requests to test size validation
        let request_result = if make_oversized_request {
            let large_body = "x".repeat(100_000); // 100KB body
            request_result.body(Body::Text(large_body))
        } else {
            request_result.body(Body::Empty)
        };
        
        // Only test if we can create a valid HTTP request
        if let Ok(request) = request_result {
            // Call the handler function
            let response_result = tokio_test::block_on(function_handler(request));
            
            // The handler should always return a response (never an error)
            prop_assert!(
                response_result.is_ok(),
                "Handler should always return a response, not an error"
            );
            
            let response = response_result.unwrap();
            
            // Verify that all required security headers are present
            let headers = response.headers();
            
            // X-Content-Type-Options: nosniff
            // Prevents MIME type sniffing attacks
            prop_assert!(
                headers.contains_key("x-content-type-options"),
                "Response missing X-Content-Type-Options header"
            );
            prop_assert_eq!(
                headers.get("x-content-type-options").unwrap().to_str().unwrap(),
                "nosniff",
                "X-Content-Type-Options header should be 'nosniff'"
            );
            
            // X-Frame-Options: DENY
            // Prevents clickjacking attacks
            prop_assert!(
                headers.contains_key("x-frame-options"),
                "Response missing X-Frame-Options header"
            );
            prop_assert_eq!(
                headers.get("x-frame-options").unwrap().to_str().unwrap(),
                "DENY",
                "X-Frame-Options header should be 'DENY'"
            );
            
            // Content-Security-Policy
            // Prevents XSS and other injection attacks
            prop_assert!(
                headers.contains_key("content-security-policy"),
                "Response missing Content-Security-Policy header"
            );
            let csp_value = headers.get("content-security-policy").unwrap().to_str().unwrap();
            prop_assert!(
                csp_value.contains("default-src 'self'"),
                "CSP should contain 'default-src self' directive"
            );
            prop_assert!(
                csp_value.contains("frame-ancestors 'none'"),
                "CSP should contain 'frame-ancestors none' directive"
            );
            
            // X-XSS-Protection: 1; mode=block
            // Enables browser XSS filtering with blocking mode
            prop_assert!(
                headers.contains_key("x-xss-protection"),
                "Response missing X-XSS-Protection header"
            );
            prop_assert_eq!(
                headers.get("x-xss-protection").unwrap().to_str().unwrap(),
                "1; mode=block",
                "X-XSS-Protection header should be '1; mode=block'"
            );
            
            // Strict-Transport-Security: max-age=31536000
            // Enforces HTTPS connections
            prop_assert!(
                headers.contains_key("strict-transport-security"),
                "Response missing Strict-Transport-Security header"
            );
            prop_assert_eq!(
                headers.get("strict-transport-security").unwrap().to_str().unwrap(),
                "max-age=31536000",
                "Strict-Transport-Security header should be 'max-age=31536000'"
            );
            
            // Verify that security headers are present regardless of response status
            // This ensures that both successful (200) and error responses (400, 405, 413)
            // include the same security protections
            let status_code = response.status().as_u16();
            prop_assert!(
                status_code == 200 || status_code == 400 || status_code == 405 || status_code == 413,
                "Response should have valid status code (200, 400, 405, or 413), got {}",
                status_code
            );
            
            // For successful responses, verify Content-Type is text/html
            if status_code == 200 {
                prop_assert!(
                    headers.contains_key("content-type"),
                    "Successful response missing Content-Type header"
                );
                prop_assert_eq!(
                    headers.get("content-type").unwrap().to_str().unwrap(),
                    "text/html",
                    "Successful response should have Content-Type: text/html"
                );
            }
            
            // For error responses, verify Content-Type is text/plain
            if status_code != 200 {
                prop_assert!(
                    headers.contains_key("content-type"),
                    "Error response missing Content-Type header"
                );
                prop_assert_eq!(
                    headers.get("content-type").unwrap().to_str().unwrap(),
                    "text/plain",
                    "Error response should have Content-Type: text/plain"
                );
            }
        }
    }
}

// Property test for security headers consistency across different response types
// 
// This test specifically focuses on ensuring that security headers are consistent
// between successful responses and various error responses.
proptest! {
    #[test]
    fn test_security_headers_consistency_property(
        // Generate scenarios that produce different response types
        scenario in prop::sample::select(vec![
            "success",           // Valid GET request -> 200 OK
            "method_not_allowed", // POST request -> 405 Method Not Allowed
            "bad_request",       // Malicious path -> 400 Bad Request
            "request_too_large", // Oversized request -> 413 Request Entity Too Large
        ]),
    ) {
        // Create request based on scenario
        let (method, path, body) = match scenario {
            "success" => ("GET", "/", Body::Empty),
            "method_not_allowed" => ("POST", "/", Body::Empty),
            "bad_request" => ("GET", "/../etc/passwd", Body::Empty),
            "request_too_large" => ("GET", "/", Body::Text("x".repeat(100_000))),
            _ => ("GET", "/", Body::Empty),
        };
        
        let request = http::Request::builder()
            .method(&method[..])
            .uri(path)
            .body(body)
            .unwrap();
        
        let response = tokio_test::block_on(function_handler(request)).unwrap();
        let headers = response.headers();
        
        // Define the complete set of required security headers
        let required_security_headers = vec![
            ("x-content-type-options", "nosniff"),
            ("x-frame-options", "DENY"),
            ("x-xss-protection", "1; mode=block"),
            ("strict-transport-security", "max-age=31536000"),
        ];
        
        // Verify all security headers are present with correct values
        for (header_name, expected_value) in required_security_headers {
            prop_assert!(
                headers.contains_key(header_name),
                "Response for scenario '{}' missing {} header",
                scenario,
                header_name
            );
            prop_assert_eq!(
                headers.get(header_name).unwrap().to_str().unwrap(),
                expected_value,
                "Response for scenario '{}' has incorrect {} header value",
                scenario,
                header_name
            );
        }
        
        // Verify CSP header is present and contains key directives
        prop_assert!(
            headers.contains_key("content-security-policy"),
            "Response for scenario '{}' missing Content-Security-Policy header",
            scenario
        );
        
        let csp_value = headers.get("content-security-policy").unwrap().to_str().unwrap();
        let required_csp_directives = vec![
            "default-src 'self'",
            "script-src 'self'",
            "style-src 'self' 'unsafe-inline'",
            "frame-ancestors 'none'",
            "base-uri 'self'",
            "form-action 'self'",
        ];
        
        for directive in required_csp_directives {
            prop_assert!(
                csp_value.contains(directive),
                "CSP for scenario '{}' missing directive: {}",
                scenario,
                directive
            );
        }
    }
}

// Property test for error handling
// 
// **Property 6: Error Handling**
// **Validates: Requirements 5.4**
// 
// This property test verifies that the system handles all error conditions correctly
// by returning appropriate error messages and logging error details.
// 
// The property being tested is:
// "For any error condition that occurs during request processing, the system should
// return appropriate error messages and log the error details"
// 
// This test generates various error scenarios and verifies that:
// 1. All errors result in proper HTTP responses (never panics or crashes)
// 2. Error responses have appropriate HTTP status codes
// 3. Error responses contain generic user messages (no information disclosure)
// 4. Error responses include request IDs for correlation
// 5. All error responses include security headers
// 6. Error details are logged internally for debugging
// 
// Feature: static-web-lambda, Property 6: Error Handling
proptest! {
    #[test]
    fn test_error_handling_property(
        // Generate different error scenarios
        error_scenario in prop::sample::select(vec![
            "invalid_method_post",      // POST request -> 405 Method Not Allowed
            "invalid_method_put",       // PUT request -> 405 Method Not Allowed
            "invalid_method_delete",    // DELETE request -> 405 Method Not Allowed
            "malicious_path_traversal", // Directory traversal -> 400 Bad Request
            "malicious_path_encoded",   // Encoded traversal -> 400 Bad Request
            "malicious_path_null_byte", // Null byte injection -> 400 Bad Request
            "oversized_request_body",   // Large request body -> 413 Request Entity Too Large
            "oversized_request_path",   // Very long path -> 400 Bad Request
            "invalid_characters",       // Dangerous characters -> 400 Bad Request
        ]),
        // Generate additional random components for more comprehensive testing
        random_path_suffix in "[a-zA-Z0-9]{0,20}",
        _random_body_size in 1000usize..200_000usize,
    ) {
        // Create request based on error scenario
        let (method, path, body, expected_status_range) = match error_scenario {
            "invalid_method_post" => {
                ("POST", format!("/{}", random_path_suffix), Body::Empty, (405, 405))
            }
            "invalid_method_put" => {
                ("PUT", format!("/{}", random_path_suffix), Body::Empty, (405, 405))
            }
            "invalid_method_delete" => {
                ("DELETE", format!("/{}", random_path_suffix), Body::Empty, (405, 405))
            }
            "malicious_path_traversal" => {
                ("GET", format!("/../../../etc/passwd/{}", random_path_suffix), Body::Empty, (400, 400))
            }
            "malicious_path_encoded" => {
                ("GET", format!("/%2e%2e/secret/{}", random_path_suffix), Body::Empty, (400, 400))
            }
            "malicious_path_null_byte" => {
                // Create path with null byte (note: this might be rejected at HTTP parsing level)
                ("GET", format!("/safe\0path/{}", random_path_suffix), Body::Empty, (400, 400))
            }
            "oversized_request_body" => {
                // Create a request body that definitely exceeds the 64KB limit
                let large_body = "x".repeat(70_000); // 70KB body, definitely over the 64KB limit
                ("GET", "/".to_string(), Body::Text(large_body), (413, 413))
            }
            "oversized_request_path" => {
                // Create very long path that exceeds the 1000 character limit in sanitize_path
                let long_path = format!("/{}", "a".repeat(1500)); // 1500+ chars, definitely over limit
                ("GET", long_path, Body::Empty, (400, 400))
            }
            "invalid_characters" => {
                ("GET", format!("/<script>alert('xss')</script>/{}", random_path_suffix), Body::Empty, (400, 400))
            }
            _ => ("GET", "/".to_string(), Body::Empty, (200, 200)), // Fallback to success case
        };
        
        // Attempt to create the HTTP request
        // Some malformed requests might fail at the HTTP parsing level
        let request_result = http::Request::builder()
            .method(&method[..])
            .uri(&path);
        
        let request_result = request_result.body(body);
        
        // Test the handler's response to the error scenario
        match request_result {
            Ok(request) => {
                // Successfully created request, test the handler
                let response_result = tokio_test::block_on(function_handler(request));
                
                // The handler should ALWAYS return a response, never an error
                // This is a critical property - the handler must be resilient
                prop_assert!(
                    response_result.is_ok(),
                    "Handler should always return a response for error scenario '{}', not an error",
                    error_scenario
                );
                
                let response = response_result.unwrap();
                let status_code = response.status().as_u16();
                
                // Verify the status code is in the expected range
                prop_assert!(
                    status_code >= expected_status_range.0 && status_code <= expected_status_range.1,
                    "Error scenario '{}' should return status code {}-{}, got {}",
                    error_scenario,
                    expected_status_range.0,
                    expected_status_range.1,
                    status_code
                );
                
                // Verify that error responses are not successful (not 2xx)
                if error_scenario != "success" {
                    prop_assert!(
                        status_code >= 400,
                        "Error scenario '{}' should return error status code (>=400), got {}",
                        error_scenario,
                        status_code
                    );
                }
                
                // Verify that all error responses include security headers
                let headers = response.headers();
                let required_security_headers = vec![
                    "x-content-type-options",
                    "x-frame-options", 
                    "x-xss-protection",
                    "strict-transport-security",
                    "content-security-policy",
                ];
                
                for header_name in required_security_headers {
                    prop_assert!(
                        headers.contains_key(header_name),
                        "Error response for scenario '{}' missing security header: {}",
                        error_scenario,
                        header_name
                    );
                }
                
                // Verify error responses have appropriate Content-Type
                if status_code >= 400 {
                    prop_assert!(
                        headers.contains_key("content-type"),
                        "Error response for scenario '{}' missing Content-Type header",
                        error_scenario
                    );
                    
                    let content_type = headers.get("content-type").unwrap().to_str().unwrap();
                    prop_assert_eq!(
                        content_type,
                        "text/plain",
                        "Error response for scenario '{}' should have Content-Type: text/plain",
                        error_scenario
                    );
                }
                
                // Verify 405 responses include Allow header
                if status_code == 405 {
                    prop_assert!(
                        headers.contains_key("allow"),
                        "405 Method Not Allowed response should include Allow header"
                    );
                    
                    let allow_header = headers.get("allow").unwrap().to_str().unwrap();
                    prop_assert!(
                        allow_header.contains("GET"),
                        "Allow header should include GET method, got: {}",
                        allow_header
                    );
                }
                
                // Verify error response body contains generic message and request ID
                if status_code >= 400 {
                    let body_bytes = match response.body() {
                        Body::Text(text) => text.as_bytes(),
                        Body::Binary(bytes) => bytes,
                        Body::Empty => &[],
                    };
                    
                    let body_text = std::str::from_utf8(body_bytes).unwrap_or("");
                    
                    // Error messages should contain a request ID for correlation
                    prop_assert!(
                        body_text.contains("Request ID:"),
                        "Error response for scenario '{}' should contain request ID, got: {}",
                        error_scenario,
                        body_text
                    );
                    
                    // Error messages should be generic (not reveal internal details)
                    // We check for specific technical terms that shouldn't appear in user messages
                    let forbidden_terms = vec![
                        "panic", "unwrap", "expect", "debug", "trace", "stack",
                        "internal error", "system error", "database", "sql", "query", 
                        "file system", "directory traversal", "config", "environment", "variable",
                        "memory", "allocation", "thread", "process", "lambda function",
                        "aws lambda", "error:", "failed:", "exception", "null pointer",
                        "sanitize", "validation failed", "security violation", "malicious",
                    ];
                    
                    let body_lower = body_text.to_lowercase();
                    for forbidden_term in forbidden_terms {
                        prop_assert!(
                            !body_lower.contains(forbidden_term),
                            "Error response for scenario '{}' contains forbidden term '{}' in message: {}",
                            error_scenario,
                            forbidden_term,
                            body_text
                        );
                    }
                    
                    // Verify message is appropriately generic based on status code
                    match status_code {
                        400 => {
                            prop_assert!(
                                body_text.contains("Bad Request") || body_text.contains("Invalid"),
                                "400 error should contain generic bad request message, got: {}",
                                body_text
                            );
                        }
                        405 => {
                            prop_assert!(
                                body_text.contains("Method Not Allowed") || body_text.contains("GET"),
                                "405 error should contain method not allowed message, got: {}",
                                body_text
                            );
                        }
                        413 => {
                            prop_assert!(
                                body_text.contains("Request Entity Too Large") || body_text.contains("size"),
                                "413 error should contain request too large message, got: {}",
                                body_text
                            );
                        }
                        500 => {
                            prop_assert!(
                                body_text.contains("Internal Server Error"),
                                "500 error should contain internal server error message, got: {}",
                                body_text
                            );
                        }
                        _ => {
                            // Other error codes should have appropriate generic messages
                            prop_assert!(
                                !body_text.is_empty(),
                                "Error response should not have empty body"
                            );
                        }
                    }
                }
                
                // Test that the response can be serialized/processed without panicking
                // This ensures the error handling is robust and doesn't cause secondary failures
                let _status_code = response.status();
                let _headers = response.headers();
                let _body = response.body();
                
                // If we reach here, the error was handled properly
                prop_assert!(true, "Error scenario '{}' handled successfully", error_scenario);
                
            }
            Err(_http_error) => {
                // Request creation failed at HTTP parsing level
                // This is acceptable for some malformed requests (like null bytes in URIs)
                // The important thing is that our handler doesn't get called with invalid requests
                prop_assert!(
                    error_scenario == "malicious_path_null_byte" || 
                    error_scenario == "oversized_request_path" ||
                    error_scenario == "invalid_characters" ||
                    path.len() > 1000,
                    "HTTP request creation should only fail for severely malformed requests, failed for scenario: {}",
                    error_scenario
                );
            }
        }
    }
}

// Property test for error message consistency and safety
// 
// This test focuses specifically on ensuring that error messages are consistent,
// safe, and don't leak sensitive information across different error types.
proptest! {
    #[test]
    fn test_error_message_safety_property(
        // Generate different types of security violations
        security_violation in prop::sample::select(vec![
            "directory_traversal",
            "encoded_traversal", 
            "null_byte_injection",
            "oversized_request",
            "invalid_method",
            "dangerous_characters",
        ]),
        // Generate random components to make attacks more realistic
        attack_payload in "[a-zA-Z0-9<>\"'&%]{0,50}",
        path_component in "[a-zA-Z0-9._-]{0,20}",
    ) {
        // Create malicious request based on violation type
        let (method, path, body) = match security_violation {
            "directory_traversal" => {
                ("GET", format!("/../../../etc/passwd/{}/{}", attack_payload, path_component), Body::Empty)
            }
            "encoded_traversal" => {
                ("GET", format!("/%2e%2e%2f{}/{}", attack_payload, path_component), Body::Empty)
            }
            "null_byte_injection" => {
                ("GET", format!("/safe\0{}/{}", attack_payload, path_component), Body::Empty)
            }
            "oversized_request" => {
                // Create a request body that definitely exceeds the 64KB limit
                let large_body = format!("{}{}", attack_payload.repeat(2000), "x".repeat(70_000)); // Ensure it's over 64KB
                ("GET", format!("/{}", path_component), Body::Text(large_body))
            }
            "invalid_method" => {
                ("POST", format!("/{}/{}", attack_payload, path_component), Body::Empty)
            }
            "dangerous_characters" => {
                ("GET", format!("/<script>{}</script>/{}", attack_payload, path_component), Body::Empty)
            }
            _ => ("GET", "/".to_string(), Body::Empty),
        };
        
        // Attempt to create and process the malicious request
        let request_result = http::Request::builder()
            .method(&method[..])
            .uri(&path)
            .body(body);
        
        if let Ok(request) = request_result {
            let response = tokio_test::block_on(function_handler(request)).unwrap();
            
            // Extract the error message from the response body
            let body_bytes = match response.body() {
                Body::Text(text) => text.as_bytes(),
                Body::Binary(bytes) => bytes,
                Body::Empty => &[],
            };
            
            let error_message = std::str::from_utf8(body_bytes).unwrap_or("");
            
            // Verify that error messages don't contain the attack payload
            // This prevents reflected XSS and information disclosure
            // Only check for potentially dangerous payloads (not single characters or numbers)
            if !attack_payload.is_empty() && attack_payload.len() > 2 && 
               (attack_payload.contains('<') || attack_payload.contains('>') || 
                attack_payload.contains('"') || attack_payload.contains('\'') ||
                attack_payload.contains("script") || attack_payload.contains("..")) {
                prop_assert!(
                    !error_message.contains(&attack_payload),
                    "Error message should not contain dangerous attack payload '{}', got message: {}",
                    attack_payload,
                    error_message
                );
            }
            
            // Verify that error messages don't contain the malicious path
            // This prevents path disclosure and log injection
            if path.len() < 100 && path != "/" && !path.is_empty() { // Only check for shorter, non-root paths to avoid false positives
                prop_assert!(
                    !error_message.contains(&path),
                    "Error message should not contain malicious path '{}', got message: {}",
                    path,
                    error_message
                );
            }
            
            // Verify that all error messages follow a consistent format
            if response.status().as_u16() >= 400 {
                // Should contain a request ID
                prop_assert!(
                    error_message.contains("Request ID:"),
                    "Error message should contain request ID, got: {}",
                    error_message
                );
                
                // Should end with a request ID in parentheses
                prop_assert!(
                    error_message.contains("(Request ID:") && error_message.ends_with(')'),
                    "Error message should end with request ID in parentheses, got: {}",
                    error_message
                );
                
                // Should not be empty
                prop_assert!(
                    !error_message.trim().is_empty(),
                    "Error message should not be empty"
                );
                
                // Should not be excessively long (prevents DoS via large error messages)
                prop_assert!(
                    error_message.len() < 500,
                    "Error message should not be excessively long, got {} characters: {}",
                    error_message.len(),
                    error_message
                );
            }
        }
    }
}
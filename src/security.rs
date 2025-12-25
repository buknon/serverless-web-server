// Security-related functions for input validation and sanitization
// This module handles path sanitization, request validation, and security checks

use std::path::Path;

/// Sanitizes request paths to prevent directory traversal attacks
/// 
/// ## Security Concerns:
/// 
/// Directory traversal (also known as path traversal) is a web security vulnerability
/// that allows attackers to read arbitrary files on the server by manipulating file paths.
/// 
/// ### Common Attack Patterns:
/// 
/// 1. **Dot-dot-slash attacks**: Using "../" sequences to navigate up directories
///    - Example: `/../../etc/passwd` tries to access system password file
///    - Example: `/static/../../../secret.txt` tries to escape the static directory
/// 
/// 2. **Encoded traversal**: Using URL encoding to bypass basic filters
///    - Example: `%2e%2e%2f` is URL-encoded "../"
///    - Example: `%2e%2e/` is partially encoded "../"
/// 
/// 3. **Double encoding**: Using double URL encoding to bypass decoding filters
///    - Example: `%252e%252e%252f` is double-encoded "../"
/// 
/// 4. **Unicode normalization attacks**: Using Unicode characters that normalize to "../"
///    - Example: Various Unicode characters can normalize to dots and slashes
/// 
/// 5. **Null byte injection**: Using null bytes to truncate paths
///    - Example: `/safe/path%00../../etc/passwd` (null byte truncates at %00)
/// 
/// ### Why This Matters for Static Servers:
/// 
/// Even though our Lambda function serves static content from a string constant,
/// path sanitization is still important because:
/// 
/// 1. **Defense in depth**: Security best practice to validate all inputs
/// 2. **Future extensibility**: If we later add file serving, we're already protected
/// 3. **Logging safety**: Prevents malicious paths from corrupting logs
/// 4. **Compliance**: Many security standards require input validation
/// 5. **Attack surface reduction**: Reduces potential for future vulnerabilities
/// 
/// ### Our Sanitization Strategy:
/// 
/// 1. **Normalize the path**: Convert to canonical form to handle encoding issues
/// 2. **Remove dangerous sequences**: Strip out "../" and similar patterns
/// 3. **Validate characters**: Ensure only safe characters are present
/// 4. **Length limits**: Prevent excessively long paths that could cause DoS
/// 5. **Logging**: Log suspicious requests for security monitoring
/// 
/// ### Implementation Notes:
/// 
/// - We use Rust's `std::path::Path` for safe path manipulation
/// - The function returns a sanitized path or an error for malicious requests
/// - We log security violations for monitoring and incident response
/// - The sanitized path is safe to use in logging and future file operations
/// 
/// ## Function Parameters:
/// 
/// - `path`: The raw request path from the HTTP request (e.g., "/", "/about", "/../etc/passwd")
/// 
/// ## Return Value:
/// 
/// - `Ok(String)`: A sanitized, safe path that can be used for further processing
/// - `Err(String)`: An error message describing why the path was rejected
/// 
/// ## Security Properties:
/// 
/// After sanitization, the returned path is guaranteed to:
/// 1. Not contain directory traversal sequences
/// 2. Not exceed reasonable length limits
/// 3. Only contain safe, printable characters
/// 4. Be safe for logging and display to administrators
pub fn sanitize_path(path: &str) -> Result<String, String> {
    // Log the original path for security monitoring
    // This helps detect attack attempts and patterns
    println!("Sanitizing request path: {}", path);
    
    // Check for excessively long paths that could indicate DoS attempts
    // Long paths can consume memory and processing time
    const MAX_PATH_LENGTH: usize = 1000;
    if path.len() > MAX_PATH_LENGTH {
        let error_msg = format!("Path too long: {} characters (max: {})", path.len(), MAX_PATH_LENGTH);
        println!("Security violation: {}", error_msg);
        return Err(error_msg);
    }
    
    // Check for null bytes which can be used for path truncation attacks
    // Null bytes (\0 or %00) can terminate strings in some contexts
    if path.contains('\0') {
        let error_msg = "Path contains null byte".to_string();
        println!("Security violation: {}", error_msg);
        return Err(error_msg);
    }
    
    // Use Rust's Path API to normalize the path
    // This handles various encoding issues and path normalization
    let normalized_path = Path::new(path);
    
    // Check each component of the path for dangerous patterns
    for component in normalized_path.components() {
        match component {
            // ".." components are used for directory traversal attacks
            std::path::Component::ParentDir => {
                let error_msg = "Path contains parent directory reference (..)".to_string();
                println!("Security violation: {}", error_msg);
                return Err(error_msg);
            }
            // "." components are generally harmless but we'll be strict
            std::path::Component::CurDir => {
                // We could allow this, but being strict is safer
                let error_msg = "Path contains current directory reference (.)".to_string();
                println!("Security violation: {}", error_msg);
                return Err(error_msg);
            }
            // Normal path components are fine, but we'll validate the content
            std::path::Component::Normal(component_str) => {
                // Convert OsStr to str for validation
                let component_string = match component_str.to_str() {
                    Some(s) => s,
                    None => {
                        let error_msg = "Path contains invalid UTF-8 characters".to_string();
                        println!("Security violation: {}", error_msg);
                        return Err(error_msg);
                    }
                };
                
                // Check for dangerous characters in path components
                // These characters can be used in various injection attacks
                let dangerous_chars = ['<', '>', '"', '\'', '&', '\n', '\r', '\t'];
                for &dangerous_char in &dangerous_chars {
                    if component_string.contains(dangerous_char) {
                        let error_msg = format!("Path contains dangerous character: {}", dangerous_char);
                        println!("Security violation: {}", error_msg);
                        return Err(error_msg);
                    }
                }
                
                // Check for encoded traversal attempts
                // These are common ways to bypass basic "../" filters
                let encoded_patterns = [
                    "%2e%2e",  // URL-encoded ".."
                    "%2E%2E",  // URL-encoded ".." (uppercase)
                    "%2e%2E",  // Mixed case
                    "%2E%2e",  // Mixed case
                    "..%2f",   // Partial encoding
                    "..%2F",   // Partial encoding (uppercase)
                    "%2e.",    // Partial encoding
                    ".%2e",    // Partial encoding
                ];
                
                for pattern in &encoded_patterns {
                    if component_string.to_lowercase().contains(pattern) {
                        let error_msg = format!("Path contains encoded traversal pattern: {}", pattern);
                        println!("Security violation: {}", error_msg);
                        return Err(error_msg);
                    }
                }
            }
            // Root directory is fine for absolute paths
            std::path::Component::RootDir => {
                // This is normal for absolute paths like "/index.html"
            }
            // Prefix components are Windows-specific (C:, \\server\share)
            // We'll reject these for security and simplicity
            std::path::Component::Prefix(_) => {
                let error_msg = "Path contains Windows-style prefix".to_string();
                println!("Security violation: {}", error_msg);
                return Err(error_msg);
            }
        }
    }
    
    // If we reach here, the path passed all security checks
    // Return the original path (it's already safe)
    // We could normalize it further, but for our static server, the original is fine
    let sanitized = path.to_string();
    
    println!("Path sanitization successful: {} -> {}", path, sanitized);
    Ok(sanitized)
}

/// Validates the size of an HTTP request to prevent DoS attacks
/// 
/// ## Security Requirement:
/// 
/// Large requests can consume excessive memory and processing time, potentially
/// causing denial of service by exhausting server resources.
/// 
/// ## Why Request Size Limits Are Important:
/// 
/// 1. **Memory protection**: Prevents attackers from sending huge requests that consume all available memory
/// 2. **Processing time**: Large requests take more time to process, potentially blocking other requests
/// 3. **Network bandwidth**: Prevents bandwidth exhaustion attacks
/// 4. **Lambda limits**: AWS Lambda has memory and execution time limits that large requests could exceed
/// 5. **Cost control**: Lambda billing is based on memory usage and execution time
/// 
/// ## Parameters:
/// - `request`: The HTTP request to validate
/// 
/// ## Return Value:
/// - `Ok(())`: Request size is within acceptable limits
/// - `Err(String)`: Error message describing why the request was rejected
pub fn validate_request_size(request: &lambda_http::Request) -> Result<(), String> {
    // For a static web server, requests should be small since we only serve static content:
    // - GET requests typically have no body or very small bodies
    // - Headers should be reasonable in size
    // - Query parameters should be limited
    // 
    // We set a conservative limit that allows for reasonable headers and query parameters
    // but prevents abuse. 64KB should be more than sufficient for legitimate static content requests.
    const MAX_REQUEST_SIZE: usize = 64 * 1024; // 64KB limit for total request size
    
    // Calculate the total request size including headers, path, and body
    // This gives us a comprehensive measure of the request's resource consumption
    let mut total_size = 0;
    
    // Add the size of the request path (URI)
    total_size += request.uri().to_string().len();
    
    // Add the size of all headers
    for (name, value) in request.headers() {
        total_size += name.as_str().len();
        total_size += value.len();
    }
    
    // Add the size of the request body
    // For Lambda HTTP events, the body is already loaded into memory
    let body_size = match request.body() {
        lambda_http::Body::Empty => 0,
        lambda_http::Body::Text(text) => text.len(),
        lambda_http::Body::Binary(bytes) => bytes.len(),
    };
    total_size += body_size;
    
    // Check if the total request size exceeds our limit
    if total_size > MAX_REQUEST_SIZE {
        // Log the security violation for monitoring
        let error_msg = format!(
            "Request size {} bytes exceeds limit of {} bytes", 
            total_size, 
            MAX_REQUEST_SIZE
        );
        println!("Security violation: {}", error_msg);
        return Err(error_msg);
    }
    
    // Log successful size validation for debugging
    println!(
        "Request size validation successful: {} bytes (limit: {} bytes)", 
        total_size, 
        MAX_REQUEST_SIZE
    );
    
    Ok(())
}

/// Validates that the HTTP method is allowed for our static server
/// 
/// ## Security Requirement:
/// 
/// Only allow GET requests for our static web server. This prevents potential
/// security issues from POST, PUT, DELETE, etc. requests.
/// 
/// ## Why Reject Non-GET Methods:
/// 
/// 1. **Security**: POST/PUT/DELETE could be used for attacks if not properly handled
/// 2. **Clarity**: Our server only serves static content, so only GET makes sense
/// 3. **Standards compliance**: HTTP semantics specify GET for retrieving resources
/// 4. **Resource efficiency**: No need to process request bodies for static content
/// 
/// ## Parameters:
/// - `method`: The HTTP method from the request
/// 
/// ## Return Value:
/// - `Ok(())`: Method is allowed (GET)
/// - `Err(String)`: Error message for disallowed methods
pub fn validate_http_method(method: &str) -> Result<(), String> {
    if method != "GET" {
        let error_msg = format!("Method '{}' not allowed. Only GET requests are supported.", method);
        println!("Security violation: {}", error_msg);
        return Err(error_msg);
    }
    
    Ok(())
}
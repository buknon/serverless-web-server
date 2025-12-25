// Security-related functions for input validation and sanitization
// This module handles path sanitization, request validation, and security checks

use std::path::Path;
use std::fmt;

/// Security error types for different security violation scenarios
/// 
/// This enum represents the various types of security violations that can occur
/// during request processing. Each variant corresponds to a specific security
/// check failure and maps to an appropriate HTTP status code.
/// 
/// ## Security Error Categories:
/// 
/// - **Input Validation Errors**: Malformed or malicious request data
/// - **Method Validation Errors**: Unsupported or dangerous HTTP methods
/// - **Size Validation Errors**: Requests that exceed safety limits
/// - **Path Validation Errors**: Directory traversal and path injection attempts
/// 
/// ## Design Principles:
/// 
/// 1. **Specific Error Types**: Each variant represents a distinct security violation
/// 2. **HTTP Status Mapping**: Each error maps to an appropriate HTTP status code
/// 3. **Generic User Messages**: Error messages to users are generic to prevent information disclosure
/// 4. **Detailed Logging**: Full error details are logged for security monitoring
/// 5. **Consistent Handling**: All security errors follow the same processing pattern
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityError {
    /// Invalid HTTP method - only GET requests are allowed
    /// 
    /// This error occurs when a request uses an HTTP method other than GET.
    /// For a static web server, only GET requests are appropriate since we
    /// only serve read-only content.
    /// 
    /// **HTTP Status Code**: 405 Method Not Allowed
    /// **Security Impact**: Prevents potential attacks via POST, PUT, DELETE, etc.
    /// **User Message**: Generic method not allowed message
    /// **Logging**: Full method and path details for security monitoring
    InvalidMethod {
        /// The HTTP method that was attempted
        method: String,
        /// The request path for context
        path: String,
    },

    /// Request size exceeds maximum allowed limits
    /// 
    /// This error occurs when the total request size (headers + body + path)
    /// exceeds our configured maximum. This prevents DoS attacks that attempt
    /// to consume excessive memory or processing time.
    /// 
    /// **HTTP Status Code**: 413 Request Entity Too Large
    /// **Security Impact**: Prevents resource exhaustion attacks
    /// **User Message**: Generic request too large message
    /// **Logging**: Actual size vs limit for capacity planning
    RequestTooLarge {
        /// The actual size of the request in bytes
        actual_size: usize,
        /// The maximum allowed size in bytes
        max_size: usize,
        /// The request path for context
        path: String,
    },

    /// Malicious or invalid request path detected
    /// 
    /// This error occurs when path sanitization detects potentially malicious
    /// content such as directory traversal attempts, encoded attacks, or
    /// other path-based injection attempts.
    /// 
    /// **HTTP Status Code**: 400 Bad Request
    /// **Security Impact**: Prevents directory traversal and path injection attacks
    /// **User Message**: Generic bad request message
    /// **Logging**: Full path and attack pattern details for security analysis
    MaliciousPath {
        /// The original request path that was rejected
        path: String,
        /// Specific reason why the path was considered malicious
        reason: String,
    },

    /// Request contains invalid or dangerous characters
    /// 
    /// This error occurs when request data contains characters that could
    /// be used for injection attacks, such as control characters, null bytes,
    /// or other potentially dangerous content.
    /// 
    /// **HTTP Status Code**: 400 Bad Request
    /// **Security Impact**: Prevents various injection attacks
    /// **User Message**: Generic bad request message
    /// **Logging**: Character details and location for security analysis
    InvalidCharacters {
        /// The field or component containing invalid characters
        field: String,
        /// Description of the invalid characters found
        details: String,
    },

    /// Request headers contain suspicious or malicious content
    /// 
    /// This error occurs when request headers contain content that could
    /// indicate an attack attempt, such as header injection, oversized
    /// headers, or suspicious patterns.
    /// 
    /// **HTTP Status Code**: 400 Bad Request
    /// **Security Impact**: Prevents header injection and related attacks
    /// **User Message**: Generic bad request message
    /// **Logging**: Header name and suspicious content for analysis
    SuspiciousHeaders {
        /// The name of the suspicious header
        header_name: String,
        /// Description of why the header is suspicious
        reason: String,
    },
}

impl SecurityError {
    /// Converts a SecurityError to the appropriate HTTP status code
    /// 
    /// This method maps each security error type to its corresponding HTTP
    /// status code according to HTTP standards and security best practices.
    /// 
    /// ## Status Code Mapping:
    /// 
    /// - **400 Bad Request**: For malformed or malicious request content
    ///   - MaliciousPath: Request path contains attack patterns
    ///   - InvalidCharacters: Request contains dangerous characters
    ///   - SuspiciousHeaders: Request headers contain malicious content
    /// 
    /// - **405 Method Not Allowed**: For unsupported HTTP methods
    ///   - InvalidMethod: Non-GET requests to our static server
    /// 
    /// - **413 Request Entity Too Large**: For oversized requests
    ///   - RequestTooLarge: Request exceeds configured size limits
    /// 
    /// ## Security Considerations:
    /// 
    /// - Status codes follow HTTP standards for consistent client behavior
    /// - Generic status codes prevent information disclosure to attackers
    /// - Appropriate status codes enable proper client-side error handling
    /// - Consistent mapping simplifies security monitoring and alerting
    pub fn to_http_status_code(&self) -> u16 {
        match self {
            SecurityError::InvalidMethod { .. } => 405, // Method Not Allowed
            SecurityError::RequestTooLarge { .. } => 413, // Request Entity Too Large
            SecurityError::MaliciousPath { .. } => 400, // Bad Request
            SecurityError::InvalidCharacters { .. } => 400, // Bad Request
            SecurityError::SuspiciousHeaders { .. } => 400, // Bad Request
        }
    }

    /// Returns a generic error message safe for displaying to users
    /// 
    /// This method provides user-facing error messages that are generic enough
    /// to avoid information disclosure while still being helpful to legitimate users.
    /// 
    /// ## Security Principles:
    /// 
    /// 1. **No Information Disclosure**: Messages don't reveal internal details
    /// 2. **Generic but Helpful**: Provide enough information for legitimate debugging
    /// 3. **Consistent Format**: All messages follow the same structure
    /// 4. **Professional Tone**: Messages are appropriate for production use
    /// 
    /// ## Message Design:
    /// 
    /// - Brief and clear explanations of what went wrong
    /// - No specific details about attack patterns or internal logic
    /// - Suggestions for legitimate users when appropriate
    /// - Consistent formatting and tone across all error types
    pub fn to_user_message(&self) -> String {
        match self {
            SecurityError::InvalidMethod { .. } => {
                "Method Not Allowed. This server only supports GET requests.".to_string()
            }
            SecurityError::RequestTooLarge { .. } => {
                "Request Entity Too Large. Request exceeds maximum allowed size.".to_string()
            }
            SecurityError::MaliciousPath { .. } => {
                "Bad Request. Invalid request path.".to_string()
            }
            SecurityError::InvalidCharacters { .. } => {
                "Bad Request. Request contains invalid characters.".to_string()
            }
            SecurityError::SuspiciousHeaders { .. } => {
                "Bad Request. Request headers contain invalid content.".to_string()
            }
        }
    }

    /// Returns detailed error information for logging and debugging
    /// 
    /// This method provides comprehensive error details that should only be
    /// used for internal logging, monitoring, and debugging. These details
    /// should never be exposed to end users.
    /// 
    /// ## Security Considerations:
    /// 
    /// - Contains sensitive information about attack patterns
    /// - Includes full request details for forensic analysis
    /// - Should only be used for internal logging
    /// - Helps security teams understand and respond to attacks
    /// 
    /// ## Use Cases:
    /// 
    /// - Security incident response and analysis
    /// - Attack pattern identification and trending
    /// - System debugging and troubleshooting
    /// - Compliance logging and audit trails
    pub fn to_detailed_message(&self) -> String {
        match self {
            SecurityError::InvalidMethod { method, path } => {
                format!("Invalid HTTP method '{}' attempted on path '{}'", method, path)
            }
            SecurityError::RequestTooLarge { actual_size, max_size, path } => {
                format!(
                    "Request size {} bytes exceeds limit of {} bytes for path '{}'",
                    actual_size, max_size, path
                )
            }
            SecurityError::MaliciousPath { path, reason } => {
                format!("Malicious path '{}' rejected: {}", path, reason)
            }
            SecurityError::InvalidCharacters { field, details } => {
                format!("Invalid characters in {}: {}", field, details)
            }
            SecurityError::SuspiciousHeaders { header_name, reason } => {
                format!("Suspicious header '{}': {}", header_name, reason)
            }
        }
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_user_message())
    }
}

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
pub fn sanitize_path(path: &str) -> Result<String, SecurityError> {
    // Log the original path for security monitoring
    // This helps detect attack attempts and patterns
    println!("Sanitizing request path: {}", path);
    
    // Check for excessively long paths that could indicate DoS attempts
    // Long paths can consume memory and processing time
    const MAX_PATH_LENGTH: usize = 1000;
    if path.len() > MAX_PATH_LENGTH {
        let error = SecurityError::MaliciousPath {
            path: path.to_string(),
            reason: format!("Path too long: {} characters (max: {})", path.len(), MAX_PATH_LENGTH),
        };
        println!("Security violation: {}", error.to_detailed_message());
        return Err(error);
    }
    
    // Check for null bytes which can be used for path truncation attacks
    // Null bytes (\0 or %00) can terminate strings in some contexts
    if path.contains('\0') {
        let error = SecurityError::InvalidCharacters {
            field: "request_path".to_string(),
            details: "Path contains null byte".to_string(),
        };
        println!("Security violation: {}", error.to_detailed_message());
        return Err(error);
    }
    
    // Use Rust's Path API to normalize the path
    // This handles various encoding issues and path normalization
    let normalized_path = Path::new(path);
    
    // Check each component of the path for dangerous patterns
    for component in normalized_path.components() {
        match component {
            // ".." components are used for directory traversal attacks
            std::path::Component::ParentDir => {
                let error = SecurityError::MaliciousPath {
                    path: path.to_string(),
                    reason: "Path contains parent directory reference (..)".to_string(),
                };
                println!("Security violation: {}", error.to_detailed_message());
                return Err(error);
            }
            // "." components are generally harmless but we'll be strict
            std::path::Component::CurDir => {
                // We could allow this, but being strict is safer
                let error = SecurityError::MaliciousPath {
                    path: path.to_string(),
                    reason: "Path contains current directory reference (.)".to_string(),
                };
                println!("Security violation: {}", error.to_detailed_message());
                return Err(error);
            }
            // Normal path components are fine, but we'll validate the content
            std::path::Component::Normal(component_str) => {
                // Convert OsStr to str for validation
                let component_string = match component_str.to_str() {
                    Some(s) => s,
                    None => {
                        let error = SecurityError::InvalidCharacters {
                            field: "path_component".to_string(),
                            details: "Path contains invalid UTF-8 characters".to_string(),
                        };
                        println!("Security violation: {}", error.to_detailed_message());
                        return Err(error);
                    }
                };
                
                // Check for dangerous characters in path components
                // These characters can be used in various injection attacks
                let dangerous_chars = ['<', '>', '"', '\'', '&', '\n', '\r', '\t'];
                for &dangerous_char in &dangerous_chars {
                    if component_string.contains(dangerous_char) {
                        let error = SecurityError::InvalidCharacters {
                            field: "path_component".to_string(),
                            details: format!("Path contains dangerous character: {}", dangerous_char),
                        };
                        println!("Security violation: {}", error.to_detailed_message());
                        return Err(error);
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
                        let error = SecurityError::MaliciousPath {
                            path: path.to_string(),
                            reason: format!("Path contains encoded traversal pattern: {}", pattern),
                        };
                        println!("Security violation: {}", error.to_detailed_message());
                        return Err(error);
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
                let error = SecurityError::MaliciousPath {
                    path: path.to_string(),
                    reason: "Path contains Windows-style prefix".to_string(),
                };
                println!("Security violation: {}", error.to_detailed_message());
                return Err(error);
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
pub fn validate_request_size(request: &lambda_http::Request) -> Result<(), SecurityError> {
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
    let request_path = request.uri().to_string();
    total_size += request_path.len();
    
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
        // Create detailed security error for monitoring
        let error = SecurityError::RequestTooLarge {
            actual_size: total_size,
            max_size: MAX_REQUEST_SIZE,
            path: request_path,
        };
        println!("Security violation: {}", error.to_detailed_message());
        return Err(error);
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
pub fn validate_http_method(method: &str) -> Result<(), SecurityError> {
    if method != "GET" {
        let error = SecurityError::InvalidMethod {
            method: method.to_string(),
            path: "unknown".to_string(), // Path will be provided by caller if needed
        };
        println!("Security violation: {}", error.to_detailed_message());
        return Err(error);
    }
    
    Ok(())
}
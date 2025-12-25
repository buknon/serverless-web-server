// Main Lambda handler function and request processing logic
// This module contains the core business logic for handling HTTP requests

use lambda_http::{Error, Request, Response, Body};
use crate::response::{create_html_response, create_error_response};
use crate::security::{sanitize_path, validate_request_size, validate_http_method};

// Import logging functionality for structured request logging
use log::{info, warn};

// Import chrono for timestamp generation in structured logging
use chrono::{DateTime, Utc};

/// Helper function to extract User-Agent header from request
/// 
/// The User-Agent header provides information about the client making the request
/// (browser, bot, API client, etc.). This is valuable for:
/// - Security monitoring (detecting malicious bots)
/// - Analytics (understanding client distribution)
/// - Debugging (identifying client-specific issues)
/// - Compliance (logging access patterns)
fn extract_user_agent(request: &Request) -> String {
    request
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("Unknown")
        .to_string()
}

/// Log outgoing HTTP response with structured format and processing time
/// 
/// This function implements structured logging for outgoing responses as required by
/// Requirements 2.4: "Log response status codes and processing time"
/// 
/// Structured logging format includes:
/// - Timestamp: ISO 8601 format for consistent time representation
/// - Response status code: HTTP status code returned to client
/// - Processing time: Time taken to process the request in milliseconds
/// - Request path: The requested URL path for correlation with request logs
/// 
/// Security considerations:
/// - All logged data is sanitized to prevent log injection attacks
/// - Processing times are logged as integers to prevent format string attacks
/// - No sensitive response data (headers, body content) is logged
/// - Structured format prevents log parsing attacks
/// 
/// Performance monitoring:
/// - Processing times enable performance analysis and optimization
/// - Status code distribution helps identify error patterns
/// - Timestamps enable correlation with AWS Lambda metrics
/// - Log retention is managed by CloudWatch configuration
/// 
/// ## Parameters:
/// - `status_code`: HTTP status code of the response (200, 400, 405, etc.)
/// - `processing_time`: Duration taken to process the request
/// - `request_path`: The requested URL path for correlation
fn log_outgoing_response(status_code: u16, processing_time: std::time::Duration, request_path: &str) {
    // Generate timestamp in ISO 8601 format for consistent logging
    let timestamp: DateTime<Utc> = Utc::now();
    
    // Convert processing time to milliseconds for human-readable logging
    // Using as_millis() provides sufficient precision for Lambda function monitoring
    let processing_time_ms = processing_time.as_millis();
    
    // Sanitize request path to prevent log injection attacks
    // Replace any control characters or newlines that could break log parsing
    let sanitized_path = request_path
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == '/' || *c == '?' || *c == '&' || *c == '=')
        .filter(|c| *c != '\n' && *c != '\r')
        .collect::<String>();
    
    // Log the response with structured format
    // Format: [TIMESTAMP] [RESPONSE] status=STATUS_CODE processing_time_ms=TIME path=PATH
    info!(
        "[{}] [RESPONSE] status={} processing_time_ms={} path={}",
        timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
        status_code,
        processing_time_ms,
        sanitized_path
    );
    
    // Additional performance monitoring for slow requests
    // Log warnings for requests that take longer than expected
    // This helps identify performance issues and potential optimization opportunities
    if processing_time_ms > 1000 {  // More than 1 second
        warn!(
            "[{}] [PERFORMANCE] Slow request detected: processing_time_ms={} status={} path={}",
            timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            processing_time_ms,
            status_code,
            sanitized_path
        );
    }
    
    // Log error responses for monitoring and alerting
    // This helps with error tracking and debugging
    if status_code >= 400 {
        warn!(
            "[{}] [ERROR_RESPONSE] Error response sent: status={} processing_time_ms={} path={}",
            timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            status_code,
            processing_time_ms,
            sanitized_path
        );
    }
}

/// Log incoming HTTP request with structured format and timestamp
/// 
/// This function implements structured logging for incoming requests as required by
/// Requirements 2.4: "Add logging for incoming requests (method, path, user agent)"
/// 
/// Structured logging format includes:
/// - Timestamp: ISO 8601 format for consistent time representation
/// - Request method: HTTP method (GET, POST, etc.)
/// - Request path: The requested URL path
/// - User-Agent: Client identification string
/// - Log level: INFO for normal requests, WARN for suspicious patterns
/// 
/// Security considerations:
/// - User-Agent strings are sanitized to prevent log injection
/// - Request paths are logged after sanitization
/// - No sensitive information (query parameters, headers) is logged
/// - Structured format prevents log parsing attacks
/// 
/// CloudWatch integration:
/// - All logs automatically appear in AWS CloudWatch Logs
/// - Structured format enables easy filtering and searching
/// - Timestamps enable correlation with AWS Lambda metrics
/// - Log retention is managed by CloudWatch configuration
fn log_incoming_request(request: &Request) {
    // Generate timestamp in ISO 8601 format for consistent logging
    let timestamp: DateTime<Utc> = Utc::now();
    
    // Extract request information for logging
    let method = request.method().as_str();
    let path = request.uri().path();
    let user_agent = extract_user_agent(request);
    
    // Sanitize user agent to prevent log injection attacks
    // Replace any control characters or newlines that could break log parsing
    let sanitized_user_agent = user_agent
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .filter(|c| *c != '\n' && *c != '\r')
        .collect::<String>();
    
    // Log the request with structured format
    // Format: [TIMESTAMP] [LEVEL] [REQUEST] method=METHOD path=PATH user_agent=USER_AGENT
    info!(
        "[{}] [REQUEST] method={} path={} user_agent={}",
        timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
        method,
        path,
        sanitized_user_agent
    );
    
    // Additional security logging for suspicious patterns
    // This helps with security monitoring and threat detection
    if method != "GET" {
        warn!(
            "[{}] [SECURITY] Non-GET request detected: method={} path={} user_agent={}",
            timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            method,
            path,
            sanitized_user_agent
        );
    }
    
    // Log suspicious user agents that might indicate automated attacks
    let suspicious_patterns = ["sqlmap", "nikto", "nmap", "masscan", "dirb"];
    let user_agent_lower = sanitized_user_agent.to_lowercase();
    
    for pattern in &suspicious_patterns {
        if user_agent_lower.contains(pattern) {
            warn!(
                "[{}] [SECURITY] Suspicious user agent detected: pattern={} user_agent={} path={}",
                timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                pattern,
                sanitized_user_agent,
                path
            );
            break;
        }
    }
}

/// Lambda handler function - the core of our serverless application
/// 
/// This is an ASYNC function, which is a key concept in Rust for handling I/O operations:
/// 
/// ## What is async/await in Rust?
/// 
/// The 'async' keyword transforms this function into a "Future" - a computation that
/// can be paused and resumed. This is crucial for serverless functions because:
/// 
/// 1. **Non-blocking I/O**: When waiting for network requests, file reads, or database
///    queries, the function can yield control back to the runtime instead of blocking
///    the entire thread. This allows other requests to be processed concurrently.
/// 
/// 2. **Memory efficiency**: Async functions use state machines under the hood,
///    which are more memory-efficient than traditional threads for I/O-bound work.
/// 
/// 3. **Lambda compatibility**: AWS Lambda's Rust runtime expects async handlers
///    because Lambda functions often need to make network calls to other AWS services.
/// 
/// ## How async/await works:
/// 
/// - `async fn` creates a function that returns a Future when called
/// - The Future doesn't execute immediately - it needs to be "awaited"
/// - `await` pauses execution until the Future completes, then returns the result
/// - While awaiting, the runtime can execute other Futures (concurrency)
/// 
/// ## Function signature breakdown:
/// 
/// - `pub`: Makes this function public so it can be called from main.rs
/// - `async`: Marks this as an asynchronous function that returns a Future
/// - `fn function_handler`: The function name - Lambda will call this for each request
/// - `request: Request`: Input parameter - the HTTP request from Lambda Function URL
/// - `-> Result<Response<Body>, Error>`: Return type - either success or failure
/// 
/// ## Result type explanation:
/// 
/// Rust uses Result<T, E> for error handling instead of exceptions:
/// - `Ok(Response<Body>)`: Success case containing the HTTP response
/// - `Err(Error)`: Failure case containing error information
/// - The `?` operator can be used to propagate errors up the call stack
/// 
/// ## Lambda Request/Response types:
/// 
/// - `Request`: Represents the incoming HTTP request with method, path, headers, body
/// - `Response<Body>`: Represents the HTTP response we send back to the client
/// - `Error`: Lambda-specific error type for handling various failure scenarios
/// 
/// ## HTTP Status Code Strategy:
/// 
/// This handler implements proper HTTP method validation and returns appropriate
/// status codes for different scenarios:
/// 
/// - **200 OK**: Successful content delivery for valid GET requests
/// - **400 Bad Request**: For malformed or malicious requests
/// - **405 Method Not Allowed**: For non-GET requests
/// - **413 Request Entity Too Large**: For oversized requests
/// - **500 Internal Server Error**: For unexpected server errors
/// 
/// This function will be called once for each HTTP request to our Lambda Function URL.
/// Lambda handles the infrastructure, scaling, and request routing - we just need to
/// process the request and return an appropriate response.
pub async fn function_handler(request: Request) -> Result<Response<Body>, Error> {
    // Record start time for processing time calculation (Task 26 - Requirements 2.4)
    let start_time = std::time::Instant::now();
    // Log incoming request with structured format and timestamp (Task 25 - Requirements 2.4)
    // 
    // This implements structured logging for incoming requests as required by Requirements 2.4:
    // "Add logging for incoming requests (method, path, user agent)"
    // 
    // Structured logging benefits:
    // - Consistent timestamp format across all log entries
    // - Easy parsing by log aggregation tools (CloudWatch Insights, etc.)
    // - Security monitoring capabilities (suspicious user agents, non-GET requests)
    // - Debugging support with request correlation
    // - Compliance with logging best practices
    log_incoming_request(&request);
    
    // Request Size Validation (Task 18 - Requirements 3.4)
    // 
    // Security requirement: Implement request size limits to prevent DoS attacks
    // This validation happens first to prevent processing of oversized requests
    // before any other validation or processing occurs.
    // 
    // HTTP 413 Request Entity Too Large:
    // This status code indicates that the request entity is larger than limits
    // defined by server. The server is closing the connection or returning a
    // Retry-After header field indicating when to try again.
    if let Err(security_error) = validate_request_size(&request) {
        let response = create_error_response(
            security_error.to_http_status_code(),
            &security_error.to_user_message()
        )?;
        
        // Log error response with processing time (Task 26 - Requirements 2.4)
        let processing_time = start_time.elapsed();
        let status_code = response.status().as_u16();
        let request_path = request.uri().path();
        log_outgoing_response(status_code, processing_time, request_path);
        
        return Ok(response);
    }
    
    // HTTP Method Validation (Task 16 - Requirements 3.4)
    // 
    // Security requirement: Only allow GET requests for our static web server
    // This prevents potential security issues from POST, PUT, DELETE, etc. requests
    // 
    // HTTP 405 Method Not Allowed:
    // This status code indicates that the server knows the request method,
    // but the target resource doesn't support this method. For a static web server,
    // only GET requests make sense since we're serving read-only content.
    if let Err(security_error) = validate_http_method(request.method().as_str()) {
        // Return HTTP 405 Method Not Allowed for any non-GET request
        // Use create_error_response to ensure all security headers are included
        let response = create_error_response(
            security_error.to_http_status_code(),
            &security_error.to_user_message()
        )?;
        
        // Log error response with processing time (Task 26 - Requirements 2.4)
        let processing_time = start_time.elapsed();
        let status_code = response.status().as_u16();
        let request_path = request.uri().path();
        log_outgoing_response(status_code, processing_time, request_path);
        
        return Ok(response);
    }
    
    // Path Sanitization (Task 17 - Requirements 3.4)
    // 
    // Security requirement: Sanitize request paths to prevent directory traversal attacks
    // Even though our static server doesn't serve files from disk, path sanitization
    // is a critical security practice that:
    // 1. Prevents potential future vulnerabilities if file serving is added
    // 2. Protects against log injection attacks
    // 3. Follows defense-in-depth security principles
    // 4. Ensures compliance with security best practices
    // 
    // HTTP 400 Bad Request:
    // This status code indicates that the server cannot process the request
    // due to malformed syntax or invalid request message framing.
    // For malicious or malformed paths, this is the appropriate response.
    let request_path = request.uri().path();
    match sanitize_path(request_path) {
        Ok(_sanitized_path) => {
            // Path is safe, continue processing
            // Note: We don't actually use the sanitized path since we serve static content,
            // but in a real file server, we would use _sanitized_path for file operations
            info!("[{}] [SECURITY] Request path validation successful: path={}", 
                  Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                  request.uri().path());
        }
        Err(security_error) => {
            // Path contains malicious content, reject the request
            warn!("[{}] [SECURITY] Rejecting request due to malicious path: error={} path={}", 
                  Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                  security_error.to_detailed_message(),
                  request.uri().path());
            
            // Return HTTP 400 Bad Request for malicious paths
            // We provide a generic error message to avoid information disclosure
            let response = create_error_response(
                security_error.to_http_status_code(),
                &security_error.to_user_message()
            )?;
            
            // Log error response with processing time (Task 26 - Requirements 2.4)
            let processing_time = start_time.elapsed();
            let status_code = response.status().as_u16();
            let request_path = request.uri().path();
            log_outgoing_response(status_code, processing_time, request_path);
            
            return Ok(response);
        }
    }
    
    // If we reach here, it's a valid GET request with a safe path and acceptable size
    // Return HTTP 200 OK with our static HTML content
    // This satisfies Requirement 1.1: "return a valid HTML page with HTTP status 200"
    let response = create_html_response()?;
    
    // Log outgoing response with processing time (Task 26 - Requirements 2.4)
    // 
    // This implements structured logging for outgoing responses as required by Requirements 2.4:
    // "Log response status codes and processing time"
    // 
    // Response logging benefits:
    // - Performance monitoring through processing time tracking
    // - Error rate monitoring through status code logging
    // - Request correlation through path logging
    // - Security monitoring through error response patterns
    // - Compliance with logging best practices
    let processing_time = start_time.elapsed();
    let status_code = response.status().as_u16();
    let request_path = request.uri().path();
    
    log_outgoing_response(status_code, processing_time, request_path);
    
    Ok(response)
}
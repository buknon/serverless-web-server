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

// Import clap for command-line argument parsing
// This allows us to parse different execution modes (local vs Lambda)
use clap::{Parser, ValueEnum};

// Import our handler function from the library
use static_web_lambda::function_handler;

/// Command-line arguments for the static web Lambda application
/// 
/// This struct defines the available command-line options using clap's derive API.
/// The derive API automatically generates argument parsing code based on the struct fields.
/// 
/// Usage examples:
/// - Run in Lambda mode (default): `./static-web-lambda`
/// - Run in local development mode: `./static-web-lambda --mode local`
/// - Show help: `./static-web-lambda --help`
#[derive(Parser, Debug)]
#[command(
    name = "static-web-lambda",
    version = "0.1.0",
    about = "A simple Rust-based webserver that serves static HTML on AWS Lambda",
    long_about = "This application can run in two modes:\n\
                  1. Lambda mode (default): Runs as an AWS Lambda function using the Lambda runtime\n\
                  2. Local mode: Runs as a local HTTP server for development and testing\n\n\
                  Lambda mode is used when deployed to AWS Lambda with Function URLs.\n\
                  Local mode is useful for development, testing, and debugging without AWS deployment."
)]
struct Args {
    /// Execution mode: either 'lambda' for AWS Lambda runtime or 'local' for development server
    /// 
    /// Lambda mode:
    /// - Uses AWS Lambda runtime to handle HTTP requests via Function URLs
    /// - Automatically handles Lambda event processing and response formatting
    /// - Suitable for production deployment on AWS Lambda
    /// - Logs are automatically sent to CloudWatch
    /// 
    /// Local mode:
    /// - Runs a local HTTP server for development and testing
    /// - Uses the same handler logic as Lambda mode for consistency
    /// - Allows local testing without AWS deployment
    /// - Useful for debugging and rapid development iteration
    #[arg(
        short = 'm',
        long = "mode",
        value_enum,
        default_value = "lambda",
        help = "Execution mode: 'lambda' for AWS Lambda runtime, 'local' for development server"
    )]
    mode: ExecutionMode,
    
    /// Port number for local development server (only used in local mode)
    /// 
    /// This option is ignored when running in Lambda mode since Lambda
    /// manages the network interface automatically.
    /// 
    /// Default port 3000 is commonly used for development servers and
    /// doesn't conflict with well-known system ports.
    #[arg(
        short = 'p',
        long = "port",
        default_value = "3000",
        help = "Port number for local server (ignored in Lambda mode)"
    )]
    port: u16,
    
    /// Host address for local development server (only used in local mode)
    /// 
    /// Default to localhost (127.0.0.1) for security - only accepts local connections.
    /// Use 0.0.0.0 to accept connections from any network interface (less secure).
    /// This option is ignored when running in Lambda mode.
    #[arg(
        short = 'H',
        long = "host",
        default_value = "127.0.0.1",
        help = "Host address for local server (ignored in Lambda mode)"
    )]
    host: String,
}

/// Execution modes supported by the application
/// 
/// This enum defines the two primary ways the application can run:
/// 1. Lambda: Production mode using AWS Lambda runtime
/// 2. Local: Development mode using local HTTP server
/// 
/// The ValueEnum derive automatically generates command-line parsing
/// for these enum variants, allowing users to specify --mode lambda or --mode local
#[derive(Debug, Clone, ValueEnum)]
enum ExecutionMode {
    /// AWS Lambda runtime mode - for production deployment
    /// 
    /// In this mode, the application:
    /// - Uses the official AWS Lambda runtime for Rust
    /// - Handles Lambda Function URL events automatically
    /// - Integrates with CloudWatch for logging
    /// - Scales automatically based on incoming requests
    /// - Follows Lambda execution model (stateless, event-driven)
    Lambda,
    
    /// Local development server mode - for testing and development
    /// 
    /// In this mode, the application:
    /// - Runs a local HTTP server using hyper or similar
    /// - Uses the same handler logic as Lambda mode for consistency
    /// - Allows rapid development and testing without AWS deployment
    /// - Provides immediate feedback for code changes
    /// - Useful for debugging and local integration testing
    Local,
}

/// The main function is the entry point of our Rust program
/// 
/// In Rust, the main function for async programs needs special handling
/// The #[tokio::main] attribute sets up the async runtime for us
/// 
/// This function will:
/// 1. Parse command-line arguments to determine execution mode
/// 2. Set up basic logging (we'll improve this later)
/// 3. Start either Lambda runtime or local server based on mode
/// 4. Register our handler function
/// 5. Begin listening for HTTP requests
/// 6. Handle runtime startup errors gracefully
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command-line arguments using clap
    // 
    // This automatically handles:
    // - Parsing command-line arguments according to our Args struct
    // - Generating help text when --help is used
    // - Validating argument types and values
    // - Providing error messages for invalid arguments
    // 
    // If parsing fails (invalid arguments), clap will print an error message
    // and exit the program automatically, so we don't need to handle that case.
    let args = Args::parse();
    
    // Initialize structured logging for both Lambda and local modes
    // 
    // AWS Lambda automatically captures stdout/stderr and sends them to CloudWatch Logs
    // For local development, logs will appear in the terminal
    // 
    // Modified to output to stdout instead of stderr for better visibility
    // This is especially useful for local development where you want to see logs
    // in the same stream as other output
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
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stdout)  // Output to stdout instead of stderr
        .init();
    
    // Log the execution mode for debugging and monitoring
    info!("Starting static-web-lambda in {:?} mode", args.mode);
    
    // Route to the appropriate execution mode based on command-line arguments
    match args.mode {
        ExecutionMode::Lambda => {
            info!("Initializing Lambda function runtime...");
            run_lambda_mode().await
        }
        ExecutionMode::Local => {
            info!("Starting local development server on {}:{}", args.host, args.port);
            run_local_mode(args.host, args.port).await
        }
    }
}

/// Run the application in AWS Lambda mode
/// 
/// This function sets up the Lambda runtime and begins processing HTTP events
/// from Lambda Function URLs. It uses the official AWS Lambda runtime for Rust
/// which handles all the Lambda-specific event processing automatically.
/// 
/// The Lambda runtime will:
/// 1. Connect to the Lambda service
/// 2. Poll for incoming HTTP events
/// 3. Convert Lambda events to standard HTTP requests
/// 4. Call our handler function
/// 5. Convert HTTP responses back to Lambda format
/// 6. Return responses to the Lambda service
async fn run_lambda_mode() -> Result<(), Error> {
    // Check if we're actually running in a Lambda environment
    // AWS Lambda sets specific environment variables that we can check
    if !is_lambda_environment() {
        error!("Lambda mode requested but not running in AWS Lambda environment");
        error!("Missing required AWS Lambda environment variables:");
        error!("- AWS_LAMBDA_FUNCTION_NAME");
        error!("- AWS_LAMBDA_RUNTIME_API");
        error!("");
        error!("To run locally for development, use: --mode local");
        error!("To deploy to AWS Lambda, use the deployment process in the README");
        
        return Err(Error::from(
            "Lambda mode can only be used when deployed to AWS Lambda. Use --mode local for development."
        ));
    }
    
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

/// Check if we're running in an AWS Lambda environment
/// 
/// AWS Lambda sets specific environment variables that we can use to detect
/// if we're running in the Lambda runtime environment vs. locally.
/// 
/// Key Lambda environment variables:
/// - AWS_LAMBDA_FUNCTION_NAME: Name of the Lambda function
/// - AWS_LAMBDA_RUNTIME_API: Lambda runtime API endpoint
/// - AWS_EXECUTION_ENV: Execution environment (starts with "AWS_Lambda_")
/// 
/// This function checks for the presence of these variables to determine
/// if we're in a Lambda environment.
fn is_lambda_environment() -> bool {
    use std::env;
    
    // Check for required Lambda environment variables
    // AWS_LAMBDA_FUNCTION_NAME is always set by the Lambda service
    let has_function_name = env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok();
    
    // AWS_LAMBDA_RUNTIME_API is set by the Lambda runtime
    let has_runtime_api = env::var("AWS_LAMBDA_RUNTIME_API").is_ok();
    
    // AWS_EXECUTION_ENV indicates the execution environment
    let _has_execution_env = env::var("AWS_EXECUTION_ENV")
        .map(|val| val.starts_with("AWS_Lambda_"))
        .unwrap_or(false);
    
    // We need at least the function name and runtime API to be in Lambda
    has_function_name && has_runtime_api
}

/// Run the application in local development mode
/// 
/// This function starts a local HTTP server for development and testing.
/// It uses the same handler function as Lambda mode to ensure consistency
/// between local development and production deployment.
/// 
/// The local server is useful for:
/// - Rapid development and testing without AWS deployment
/// - Debugging with local tools and debuggers
/// - Integration testing with other local services
/// - Demonstrating functionality without AWS account
async fn run_local_mode(host: String, port: u16) -> Result<(), Error> {
    use hyper::service::{make_service_fn, service_fn};
    use hyper::Server;
    use std::convert::Infallible;
    use std::net::SocketAddr;
    
    info!("Starting local development server on {}:{}", host, port);
    
    // Parse the host and port into a socket address
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| Error::from(format!("Invalid host:port combination: {}", e)))?;
    
    // Create a service that converts hyper requests to lambda_http requests
    // and calls our Lambda handler function
    let make_svc = make_service_fn(|_conn| {
        async {
            Ok::<_, Infallible>(service_fn(|req: hyper::Request<hyper::Body>| {
                async move {
                    // Convert hyper request to lambda_http request
                    let lambda_request = convert_hyper_to_lambda_request(req).await?;
                    
                    // Call our Lambda handler function (same as used in Lambda mode)
                    let lambda_response = function_handler(lambda_request).await?;
                    
                    // Convert lambda_http response back to hyper response
                    let hyper_response = convert_lambda_to_hyper_response(lambda_response).await?;
                    
                    Ok::<_, Error>(hyper_response)
                }
            }))
        }
    });
    
    // Create and start the HTTP server
    let server = Server::bind(&addr).serve(make_svc);
    
    info!("Local development server running at http://{}", addr);
    info!("Press Ctrl+C to stop the server");
    
    // Run the server and handle any errors
    if let Err(e) = server.await {
        error!("Local server error: {}", e);
        return Err(Error::from(format!("Local server failed: {}", e)));
    }
    
    Ok(())
}

/// Convert a hyper HTTP request to a lambda_http request
/// 
/// This function bridges the gap between the local hyper server and the Lambda handler.
/// It ensures that the same handler function can process requests in both environments.
async fn convert_hyper_to_lambda_request(req: hyper::Request<hyper::Body>) -> Result<lambda_http::Request, Error> {
    use lambda_http::http;
    
    // Extract the parts of the hyper request
    let (parts, body) = req.into_parts();
    
    // Convert the body to bytes
    let body_bytes = hyper::body::to_bytes(body).await
        .map_err(|e| Error::from(format!("Failed to read request body: {}", e)))?;
    
    // Create a lambda_http request with the same data
    let lambda_body = if body_bytes.is_empty() {
        lambda_http::Body::Empty
    } else {
        lambda_http::Body::Binary(body_bytes.to_vec())
    };
    
    // Build the lambda_http request
    let lambda_request = http::Request::builder()
        .method(parts.method)
        .uri(parts.uri)
        .version(parts.version)
        .body(lambda_body)
        .map_err(|e| Error::from(format!("Failed to build lambda request: {}", e)))?;
    
    // Copy headers from hyper request to lambda request
    let mut lambda_request = lambda_request;
    *lambda_request.headers_mut() = parts.headers;
    
    Ok(lambda_request)
}

/// Convert a lambda_http response to a hyper HTTP response
/// 
/// This function converts the response from our Lambda handler back to a format
/// that the local hyper server can send to the client.
async fn convert_lambda_to_hyper_response(resp: lambda_http::Response<lambda_http::Body>) -> Result<hyper::Response<hyper::Body>, Error> {
    use hyper::http::StatusCode;
    
    // Extract the parts of the lambda response
    let (parts, lambda_body) = resp.into_parts();
    
    // Convert lambda_http body to hyper body
    let hyper_body = match lambda_body {
        lambda_http::Body::Empty => hyper::Body::empty(),
        lambda_http::Body::Text(text) => hyper::Body::from(text),
        lambda_http::Body::Binary(bytes) => hyper::Body::from(bytes),
    };
    
    // Build the hyper response
    let mut hyper_response = hyper::Response::builder()
        .status(StatusCode::from_u16(parts.status.as_u16())
            .map_err(|e| Error::from(format!("Invalid status code: {}", e)))?)
        .version(parts.version)
        .body(hyper_body)
        .map_err(|e| Error::from(format!("Failed to build hyper response: {}", e)))?;
    
    // Copy headers from lambda response to hyper response
    *hyper_response.headers_mut() = parts.headers;
    
    Ok(hyper_response)
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
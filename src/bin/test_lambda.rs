// Test binary to simulate a Lambda call and test the actual handler function
// This calls the real handler function to ensure we're testing the actual Lambda logic

use lambda_http::{Error, Body};
use tokio;

// Import the actual handler function from the library
use static_web_lambda::function_handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create a mock HTTP GET request (same as what Lambda would receive)
    let request = lambda_http::http::Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::Empty)
        .map_err(|e| Error::from(format!("Request build error: {}", e)))?;
    
    // Call the actual Lambda handler function
    let response = function_handler(request).await?;
    
    // Extract and print the HTML content
    let body_bytes = hyper::body::to_bytes(response.into_body()).await
        .map_err(|e| Error::from(format!("Body conversion error: {}", e)))?;
    let html_content = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| Error::from(format!("UTF-8 conversion error: {}", e)))?;
    
    // Output the actual HTML content produced by the handler
    println!("{}", html_content);
    
    Ok(())
}
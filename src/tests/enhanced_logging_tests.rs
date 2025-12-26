// Unit tests for enhanced error logging functionality (Task 30)
// These tests verify the enhanced error logging implementation

use crate::response::{create_generic_error_response, ApplicationError};
use crate::security::SecurityError;
use std::env;

#[cfg(test)]
mod enhanced_logging_tests {
    use super::*;

    /// Test that request IDs are generated consistently
    #[tokio::test]
    async fn test_request_id_generation() {
        // Test with AWS Lambda environment variables
        env::set_var("_X_AMZN_TRACE_ID", "Root=1-5e1b4151-5ac6c58f5b5dcc1e1e0a7e1c;Parent=123;Sampled=1");
        
        // Create an error to trigger request ID generation
        let error = ApplicationError::Security {
            security_error: SecurityError::InvalidMethod {
                method: "POST".to_string(),
                path: "/test".to_string(),
            },
            context: "test context".to_string(),
        };
        
        let response = create_generic_error_response(error).unwrap();
        
        // Check that response contains a request ID
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_content = String::from_utf8(body_bytes.to_vec()).unwrap();
        
        assert!(
            body_content.contains("(Request ID: "),
            "Response should contain request ID. Got: {}",
            body_content
        );
        
        // Clean up environment variable
        env::remove_var("_X_AMZN_TRACE_ID");
    }

    /// Test that error type names are correctly categorized
    #[test]
    fn test_error_type_categorization() {
        let security_error = ApplicationError::Security {
            security_error: SecurityError::InvalidMethod {
                method: "POST".to_string(),
                path: "/".to_string(),
            },
            context: "test".to_string(),
        };
        assert_eq!(security_error.error_type_name(), "Security");

        let internal_error = ApplicationError::InternalError {
            details: "Test error".to_string(),
            cause: None,
        };
        assert_eq!(internal_error.error_type_name(), "Internal");

        let request_error = ApplicationError::RequestError {
            details: "Test error".to_string(),
            component: "test".to_string(),
        };
        assert_eq!(request_error.error_type_name(), "Request");

        let service_error = ApplicationError::ServiceUnavailable {
            reason: "Test".to_string(),
            retry_after: None,
        };
        assert_eq!(service_error.error_type_name(), "ServiceUnavailable");
    }

    /// Test that detailed error messages contain full context
    #[test]
    fn test_detailed_error_messages() {
        let security_error = ApplicationError::Security {
            security_error: SecurityError::MaliciousPath {
                path: "/../etc/passwd".to_string(),
                reason: "Directory traversal attempt".to_string(),
            },
            context: "path sanitization".to_string(),
        };

        let detailed_msg = security_error.to_detailed_message();
        
        assert!(
            detailed_msg.contains("Security Error in path sanitization"),
            "Detailed message should contain context. Got: {}",
            detailed_msg
        );
        assert!(
            detailed_msg.contains("/../etc/passwd"),
            "Detailed message should contain path. Got: {}",
            detailed_msg
        );
        assert!(
            detailed_msg.contains("Directory traversal attempt"),
            "Detailed message should contain reason. Got: {}",
            detailed_msg
        );
    }

    /// Test that user messages remain generic and safe
    #[test]
    fn test_generic_user_messages() {
        let security_error = ApplicationError::Security {
            security_error: SecurityError::MaliciousPath {
                path: "/../etc/passwd".to_string(),
                reason: "Directory traversal attempt".to_string(),
            },
            context: "path sanitization".to_string(),
        };

        let user_msg = security_error.to_generic_user_message();
        
        // Should be generic
        assert_eq!(user_msg, "Bad Request. Invalid request path.");
        
        // Should NOT contain sensitive details
        assert!(!user_msg.contains("/../etc/passwd"));
        assert!(!user_msg.contains("Directory traversal"));
        assert!(!user_msg.contains("path sanitization"));
    }

    /// Test that error responses include request IDs for correlation
    #[tokio::test]
    async fn test_error_response_includes_request_id() {
        let error = ApplicationError::InternalError {
            details: "Test internal error".to_string(),
            cause: Some("Test cause".to_string()),
        };

        let response = create_generic_error_response(error).unwrap();
        
        // Check status code
        assert_eq!(response.status(), 500);
        
        // Check that response body includes request ID
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_content = String::from_utf8(body_bytes.to_vec()).unwrap();
        
        assert!(
            body_content.starts_with("Internal Server Error. Please try again later."),
            "Response should start with generic message. Got: {}",
            body_content
        );
        
        assert!(
            body_content.contains("(Request ID: "),
            "Response should contain request ID. Got: {}",
            body_content
        );
        
        // Should end with closing parenthesis
        assert!(
            body_content.ends_with(")"),
            "Response should end with closing parenthesis. Got: {}",
            body_content
        );
    }

    /// Test different request ID sources
    #[tokio::test]
    async fn test_request_id_sources() {
        // Test X-Ray trace ID
        env::set_var("_X_AMZN_TRACE_ID", "Root=1-test-trace-id;Parent=123");
        let error1 = ApplicationError::RequestError {
            details: "Test".to_string(),
            component: "test".to_string(),
        };
        let response1 = create_generic_error_response(error1).unwrap();
        let body_bytes1 = hyper::body::to_bytes(response1.into_body()).await.unwrap();
        let body_content1 = String::from_utf8(body_bytes1.to_vec()).unwrap();
        assert!(body_content1.contains("(Request ID: trace-"));
        
        // Clean up and test Lambda request ID
        env::remove_var("_X_AMZN_TRACE_ID");
        env::set_var("AWS_LAMBDA_REQUEST_ID", "test-lambda-request-id");
        let error2 = ApplicationError::RequestError {
            details: "Test".to_string(),
            component: "test".to_string(),
        };
        let response2 = create_generic_error_response(error2).unwrap();
        let body_bytes2 = hyper::body::to_bytes(response2.into_body()).await.unwrap();
        let body_content2 = String::from_utf8(body_bytes2.to_vec()).unwrap();
        assert!(body_content2.contains("(Request ID: lambda-"));
        
        // Clean up and test log stream
        env::remove_var("AWS_LAMBDA_REQUEST_ID");
        env::set_var("AWS_LAMBDA_LOG_STREAM_NAME", "/aws/lambda/function/2023/12/25/stream-id");
        let error3 = ApplicationError::RequestError {
            details: "Test".to_string(),
            component: "test".to_string(),
        };
        let response3 = create_generic_error_response(error3).unwrap();
        let body_bytes3 = hyper::body::to_bytes(response3.into_body()).await.unwrap();
        let body_content3 = String::from_utf8(body_bytes3.to_vec()).unwrap();
        assert!(body_content3.contains("(Request ID: stream-"));
        
        // Clean up
        env::remove_var("AWS_LAMBDA_LOG_STREAM_NAME");
    }
}
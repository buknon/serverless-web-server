# AWS Lambda Function for Static Web Server
# This file defines the Lambda function resource that runs the Rust-based static web server
# The function uses the custom runtime (provided.al2) to run compiled Rust binaries

# Lambda Function Resource
# This creates the actual Lambda function that will serve static HTML content
resource "aws_lambda_function" "static_web_lambda" {
  # Function configuration
  function_name = var.function_name
  description   = "Rust-based static web server serving HTML content via Lambda Function URL"

  # Runtime configuration
  # Using provided.al2 runtime for custom Rust binaries
  # The binary must be named 'bootstrap' for custom runtimes
  runtime = "provided.al2"
  
  # Handler is not used for custom runtimes (provided.al2)
  # The bootstrap binary serves as the entry point
  handler = "bootstrap"

  # Architecture - using x86_64 for compatibility
  # arm64 is also supported and may be more cost-effective
  architectures = ["x86_64"]

  # Deployment package
  # References the ZIP file created by the build script
  # The ZIP must contain a 'bootstrap' executable at the root level
  filename         = var.deployment_package_path
  source_code_hash = filebase64sha256(var.deployment_package_path)

  # Resource allocation
  memory_size = var.lambda_memory  # Memory in MB (also affects CPU allocation)
  timeout     = var.lambda_timeout # Maximum execution time in seconds

  # IAM role for execution permissions
  # References the role created in iam.tf
  role = aws_iam_role.lambda_execution_role.arn

  # Environment variables (if needed)
  # Currently empty as static content doesn't require configuration
  environment {
    variables = {
      RUST_LOG = "info"  # Set log level for Rust applications
      # Add other environment variables here if needed
    }
  }

  # Concurrency configuration
  # Controls how many instances can run simultaneously
  # Only set reserved_concurrent_executions if it's not -1 (unreserved)
  reserved_concurrent_executions = var.reserved_concurrency != -1 ? var.reserved_concurrency : null

  # Dead letter queue configuration (optional)
  # Uncomment if you want to capture failed invocations
  # dead_letter_config {
  #   target_arn = aws_sqs_queue.lambda_dlq.arn
  # }

  # VPC configuration (not needed for this use case)
  # Uncomment if Lambda needs to access VPC resources
  # vpc_config {
  #   subnet_ids         = var.subnet_ids
  #   security_group_ids = var.security_group_ids
  # }

  # Tracing configuration for AWS X-Ray (optional)
  # Uncomment to enable distributed tracing
  # tracing_config {
  #   mode = "Active"
  # }

  # Logging configuration
  # Ensures logs go to the CloudWatch log group we created
  logging_config {
    log_format = "Text"  # Use "JSON" for structured logging
    log_group  = aws_cloudwatch_log_group.lambda_logs.name
  }

  # Tags for resource organization and cost tracking
  tags = merge(var.tags, {
    Name        = var.function_name
    Description = "Static web server Lambda function serving HTML content"
    Component   = "compute"
    Environment = var.environment
    Runtime     = "provided.al2"
    Language    = "rust"
  })

  # Dependencies
  # Ensure IAM role and log group exist before creating function
  depends_on = [
    aws_iam_role_policy_attachment.lambda_basic_execution,
    aws_iam_role_policy_attachment.lambda_custom_policy_attachment,
    aws_cloudwatch_log_group.lambda_logs
  ]
}

# Lambda Function URL (if enabled)
# Provides direct HTTP access to the Lambda function without API Gateway
resource "aws_lambda_function_url" "static_web_url" {
  count = var.enable_function_url ? 1 : 0

  function_name      = aws_lambda_function.static_web_lambda.function_name
  authorization_type = "NONE"  # Public access for static content

  # CORS configuration for browser compatibility
  # Restricts cross-origin requests to enhance security while maintaining functionality
  cors {
    allow_credentials = false                    # No credentials needed for static content (security best practice)
    allow_origins     = var.cors_allowed_origins # Configurable origins - restrict in production for better security
    allow_methods     = ["GET"]                  # Only GET requests for static content (meets requirement 3.4)
    allow_headers     = ["date", "keep-alive", "content-type", "x-amz-date", "authorization", "x-api-key", "x-amz-security-token"]
    expose_headers    = ["date", "keep-alive"]   # Headers that browsers can access
    max_age          = var.cors_max_age         # Cache preflight responses to reduce overhead
  }

  # Note: aws_lambda_function_url resource does not support tags
}

# Lambda Permission for Function URL (automatically created, but explicit is better)
# This permission allows the Function URL to invoke the Lambda function
resource "aws_lambda_permission" "allow_function_url" {
  count = var.enable_function_url ? 1 : 0

  statement_id           = "AllowExecutionFromFunctionURL"
  action                = "lambda:InvokeFunctionUrl"
  function_name         = aws_lambda_function.static_web_lambda.function_name
  principal             = "*"  # Public access for static content
  function_url_auth_type = "NONE"
}

# Note: Outputs have been moved to outputs.tf for centralized management
# This keeps all outputs in one place for better organization

# Lambda Function Configuration Notes:
# 1. Runtime: provided.al2 for custom Rust binaries
#    - Requires 'bootstrap' executable in deployment package
#    - More control over runtime environment
#    - Better performance for compiled languages
#
# 2. Memory and Timeout:
#    - 128MB memory is sufficient for static content serving
#    - 30-second timeout provides buffer for cold starts
#    - Memory affects CPU allocation (more memory = more CPU)
#
# 3. Architecture: x86_64 for broad compatibility
#    - arm64 (Graviton2) may offer better price/performance
#    - Ensure build process targets correct architecture
#
# 4. Deployment Package:
#    - Must be ZIP file containing 'bootstrap' executable
#    - Source code hash triggers updates when package changes
#    - Build process should create this package
#
# 5. Function URL vs API Gateway:
#    - Function URL: Simpler, lower cost, direct HTTP access
#    - API Gateway: More features (caching, throttling, custom domains)
#    - Function URL is perfect for simple static content serving
#
# 6. Security Considerations:
#    - Public access (authorization_type = "NONE") for static content
#    - CORS configured for browser compatibility
#    - Security headers should be set in application code
#    - No VPC configuration needed for public static content
#
# 7. Monitoring and Logging:
#    - CloudWatch Logs automatically configured
#    - X-Ray tracing available if needed
#    - CloudWatch metrics automatically collected
#
# This configuration creates a production-ready Lambda function for serving
# static HTML content with proper security, monitoring, and cost optimization.
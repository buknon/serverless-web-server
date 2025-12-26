# Terraform Outputs for Static Web Lambda
# This file defines all outputs that provide useful information after deployment
# These outputs help users access and manage the deployed resources

# Primary Function URL Output
# This is the main output users need to access the deployed static web server
output "function_url" {
  description = "Public HTTPS URL of the Lambda Function URL for accessing the static web server"
  value       = var.enable_function_url ? aws_lambda_function_url.static_web_url[0].function_url : null
  sensitive   = false
}

output "function_url_id" {
  description = "ID of the Lambda Function URL (if enabled)"
  value       = var.enable_function_url ? aws_lambda_function_url.static_web_url[0].url_id : null
  sensitive   = false
}

# Lambda Function Information
# Essential information about the deployed Lambda function
output "function_name" {
  description = "Name of the deployed Lambda function"
  value       = aws_lambda_function.static_web_lambda.function_name
  sensitive   = false
}

output "function_arn" {
  description = "ARN of the deployed Lambda function"
  value       = aws_lambda_function.static_web_lambda.arn
  sensitive   = false
}

output "function_invoke_arn" {
  description = "Invoke ARN of the Lambda function (for API Gateway integration)"
  value       = aws_lambda_function.static_web_lambda.invoke_arn
  sensitive   = false
}

# CloudWatch Log Group Information
# Useful for monitoring and debugging the deployed function
output "log_group_name" {
  description = "Name of the CloudWatch log group for Lambda function logs"
  value       = aws_cloudwatch_log_group.lambda_logs.name
  sensitive   = false
}

output "log_group_arn" {
  description = "ARN of the CloudWatch log group for Lambda function logs"
  value       = aws_cloudwatch_log_group.lambda_logs.arn
  sensitive   = false
}

# AWS Console Links
# Direct links to AWS console for easy management
output "lambda_console_url" {
  description = "Direct link to Lambda function in AWS Console"
  value       = "https://${var.aws_region}.console.aws.amazon.com/lambda/home?region=${var.aws_region}#/functions/${aws_lambda_function.static_web_lambda.function_name}"
  sensitive   = false
}

output "cloudwatch_logs_url" {
  description = "Direct link to CloudWatch logs in AWS Console"
  value       = "https://${var.aws_region}.console.aws.amazon.com/cloudwatch/home?region=${var.aws_region}#logsV2:log-groups/log-group/${replace(aws_cloudwatch_log_group.lambda_logs.name, "/", "$252F")}"
  sensitive   = false
}

# Deployment Summary
# Comprehensive information about the deployment
output "deployment_summary" {
  description = "Summary of deployed resources and access information"
  value = {
    function_name    = aws_lambda_function.static_web_lambda.function_name
    function_url     = var.enable_function_url ? aws_lambda_function_url.static_web_url[0].function_url : "Function URL not enabled"
    log_group        = aws_cloudwatch_log_group.lambda_logs.name
    aws_region       = var.aws_region
    environment      = var.environment
    memory_mb        = var.lambda_memory
    timeout_seconds  = var.lambda_timeout
    runtime          = "provided.al2"
    architecture     = "x86_64"
  }
  sensitive = false
}

# Function URL Details (if enabled)
# Additional information about the Function URL configuration
output "function_url_details" {
  description = "Detailed information about the Lambda Function URL configuration"
  value = var.enable_function_url ? {
    url                = aws_lambda_function_url.static_web_url[0].function_url
    url_id             = aws_lambda_function_url.static_web_url[0].url_id
    authorization_type = aws_lambda_function_url.static_web_url[0].authorization_type
    cors_config = {
      allow_origins  = aws_lambda_function_url.static_web_url[0].cors[0].allow_origins
      allow_methods  = aws_lambda_function_url.static_web_url[0].cors[0].allow_methods
      allow_headers  = aws_lambda_function_url.static_web_url[0].cors[0].allow_headers
      expose_headers = aws_lambda_function_url.static_web_url[0].cors[0].expose_headers
      max_age        = aws_lambda_function_url.static_web_url[0].cors[0].max_age
    }
  } : null
  sensitive = false
}

# Resource Identifiers
# Useful for integration with other Terraform configurations or scripts
output "resource_ids" {
  description = "Collection of AWS resource identifiers for integration purposes"
  value = {
    lambda_function_name = aws_lambda_function.static_web_lambda.function_name
    lambda_function_arn  = aws_lambda_function.static_web_lambda.arn
    log_group_name       = aws_cloudwatch_log_group.lambda_logs.name
    log_group_arn        = aws_cloudwatch_log_group.lambda_logs.arn
    iam_role_arn         = aws_iam_role.lambda_execution_role.arn
    function_url_id      = var.enable_function_url ? aws_lambda_function_url.static_web_url[0].url_id : null
  }
  sensitive = false
}

# Output Usage Instructions
# These outputs provide the following information after deployment:
#
# 1. function_url: The primary URL to access your static web server
#    - Copy this URL and paste it into your browser to view the static HTML page
#    - This is the main output you'll use to share your deployed application
#
# 2. function_name: The name of your Lambda function in AWS
#    - Use this to identify your function in the AWS Console
#    - Needed for AWS CLI commands or API calls
#
# 3. log_group_name: CloudWatch log group for debugging
#    - View logs in AWS Console to troubleshoot issues
#    - Monitor request patterns and performance
#
# 4. lambda_console_url: Direct link to Lambda function in AWS Console
#    - Click this link to manage your function settings
#    - View metrics, configuration, and monitoring information
#
# 5. cloudwatch_logs_url: Direct link to CloudWatch logs
#    - Click this link to view function execution logs
#    - Debug errors and monitor application behavior
#
# 6. deployment_summary: Complete overview of deployed resources
#    - Comprehensive information about your deployment
#    - Useful for documentation and team sharing
#
# Example usage after deployment:
# $ terraform output function_url
# $ terraform output -json deployment_summary
# $ terraform output lambda_console_url
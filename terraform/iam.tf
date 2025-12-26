# IAM Role and Policies for Static Web Lambda
# This file defines the IAM role and policies required for the Lambda function to execute
# Following the principle of least privilege - only granting the minimum permissions needed

# IAM Role for Lambda Execution
# This role allows AWS Lambda service to assume it and execute the function
resource "aws_iam_role" "lambda_execution_role" {
  name = "${var.function_name}-execution-role"

  # Trust policy - defines which AWS services can assume this role
  # Only the Lambda service is allowed to assume this role
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        # Allow AWS Lambda service to assume this role
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
        # No conditions - Lambda service can always assume this role
        # In production, you might add conditions like source account or VPC
      }
    ]
  })

  # Tags for resource organization and cost tracking
  tags = merge(var.tags, {
    Name        = "${var.function_name}-execution-role"
    Description = "IAM role for Lambda function execution with minimal required permissions"
    Component   = "iam"
  })
}

# Attach AWS Managed Policy for Basic Lambda Execution
# This policy provides the minimum permissions needed for Lambda to execute:
# - Create and write to CloudWatch Logs
# - Create log groups and log streams
resource "aws_iam_role_policy_attachment" "lambda_basic_execution" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# Custom inline policy for additional permissions specific to our function
# This policy grants only the specific permissions our static web Lambda needs
resource "aws_iam_policy" "lambda_custom_policy" {
  name        = "${var.function_name}-custom-policy"
  description = "Custom policy for static web Lambda with minimal required permissions"

  # Policy document defining specific permissions
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        # CloudWatch Logs permissions for our specific log group
        # This is more restrictive than the basic execution role
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",  # Create log group if it doesn't exist
          "logs:CreateLogStream", # Create log streams within the group
          "logs:PutLogEvents"     # Write log events to streams
        ]
        # Restrict to only our function's log group
        Resource = [
          "arn:aws:logs:${var.aws_region}:*:log-group:/aws/lambda/${var.function_name}",
          "arn:aws:logs:${var.aws_region}:*:log-group:/aws/lambda/${var.function_name}:*"
        ]
      }
    ]
  })

  tags = merge(var.tags, {
    Name        = "${var.function_name}-custom-policy"
    Description = "Custom IAM policy with minimal permissions for static web Lambda"
    Component   = "iam"
  })
}

# Attach the custom policy to the Lambda execution role
resource "aws_iam_role_policy_attachment" "lambda_custom_policy_attachment" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = aws_iam_policy.lambda_custom_policy.arn
}

# Output the IAM role ARN for use by other resources
# This will be referenced by the Lambda function resource
output "lambda_execution_role_arn" {
  description = "ARN of the IAM role for Lambda execution"
  value       = aws_iam_role.lambda_execution_role.arn
}

# Output the IAM role name for reference
output "lambda_execution_role_name" {
  description = "Name of the IAM role for Lambda execution"
  value       = aws_iam_role.lambda_execution_role.name
}

# Security Notes:
# 1. This role follows the principle of least privilege
# 2. Only Lambda service can assume this role (no cross-account access)
# 3. CloudWatch Logs permissions are restricted to our specific log group
# 4. No permissions for other AWS services (S3, DynamoDB, etc.) unless needed
# 5. No wildcard permissions - all resources are explicitly specified
# 6. Tags are applied for proper resource management and cost tracking
#
# Permissions Granted:
# - Basic Lambda execution (from AWS managed policy)
# - CloudWatch Logs creation and writing (scoped to our function)
#
# Permissions NOT Granted (following least privilege):
# - No S3 access (static content is embedded in code)
# - No database access (no data persistence needed)
# - No network access beyond basic Lambda networking
# - No access to other AWS services
# - No cross-account access
# - No administrative permissions
#
# This role is sufficient for a static web server that:
# - Serves embedded HTML content
# - Logs requests and responses
# - Runs in AWS Lambda environment
# - Uses Lambda Function URLs for HTTP access
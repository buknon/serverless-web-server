# CloudWatch Log Group for Static Web Lambda
# This file defines the CloudWatch log group for Lambda function logs
# Proper log management is essential for debugging and monitoring serverless applications

# CloudWatch Log Group for Lambda Function
# AWS Lambda automatically creates log groups, but explicitly defining them provides better control
# over retention policies, permissions, and resource management
resource "aws_cloudwatch_log_group" "lambda_logs" {
  # Log group name follows AWS Lambda convention: /aws/lambda/{function-name}
  # This ensures Lambda runtime can write to the correct log group
  name = "/aws/lambda/${var.function_name}"

  # Retention period for log events
  # Balances debugging needs with storage costs
  # Common values: 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365 days
  retention_in_days = var.log_retention_days

  # Skip deletion protection for development environments
  # In production, consider setting this to true to prevent accidental deletion
  skip_destroy = false

  # Tags for resource organization and cost tracking
  tags = merge(var.tags, {
    Name        = "${var.function_name}-logs"
    Description = "CloudWatch log group for Lambda function logs with ${var.log_retention_days} day retention"
    Component   = "logging"
    Environment = var.environment
  })
}

# Note: Outputs have been moved to outputs.tf for centralized management
# This keeps all outputs in one place for better organization

# Log Management Notes:
# 1. Log Retention: Configured via variable (default 14 days)
#    - Shorter retention (1-7 days): Lower cost, good for development
#    - Medium retention (14-30 days): Good balance for most applications
#    - Longer retention (90+ days): Higher cost, good for compliance/audit needs
#
# 2. Log Group Naming: Follows AWS Lambda convention
#    - Pattern: /aws/lambda/{function-name}
#    - Lambda runtime automatically writes to this location
#    - Must match exactly for automatic log delivery
#
# 3. Permissions: IAM role in iam.tf grants necessary permissions
#    - logs:CreateLogGroup (if log group doesn't exist)
#    - logs:CreateLogStream (for individual execution streams)
#    - logs:PutLogEvents (for writing log entries)
#
# 4. Cost Considerations:
#    - CloudWatch Logs charges for ingestion and storage
#    - Retention period directly affects storage costs
#    - Monitor log volume in production environments
#
# 5. Security:
#    - Log group is created in the same region as Lambda function
#    - Access controlled via IAM policies (least privilege)
#    - No public access - logs are private to AWS account
#
# 6. Monitoring Integration:
#    - CloudWatch Insights can query these logs
#    - CloudWatch Alarms can monitor log patterns
#    - AWS X-Ray tracing can correlate with log entries
#
# This log group will capture:
# - Lambda runtime logs (cold starts, timeouts, errors)
# - Application logs from Rust code (println!, log crate output)
# - Request/response information for debugging
# - Security events and error conditions
# - Performance metrics and timing information
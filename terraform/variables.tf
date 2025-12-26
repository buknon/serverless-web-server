# Terraform Variables for Static Web Lambda
# This file defines all configurable values for the infrastructure deployment

variable "function_name" {
  description = "Name of the AWS Lambda function. This will be used as the function identifier in AWS."
  type        = string
  default     = "static-web-lambda"

  validation {
    condition     = can(regex("^[a-zA-Z0-9-_]+$", var.function_name))
    error_message = "Function name must contain only alphanumeric characters, hyphens, and underscores."
  }
}

variable "aws_region" {
  description = "AWS region where the Lambda function and related resources will be deployed. Choose a region close to your users for better performance."
  type        = string
  default     = "us-east-1"

  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.aws_region))
    error_message = "AWS region must be a valid region identifier (e.g., us-east-1, eu-west-1)."
  }
}

variable "environment" {
  description = "Environment name for resource tagging and identification (e.g., dev, staging, prod). Used for organizing resources and cost tracking."
  type        = string
  default     = "dev"

  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be one of: dev, staging, prod."
  }
}

variable "log_retention_days" {
  description = "Number of days to retain CloudWatch logs for the Lambda function. Longer retention increases storage costs but provides more debugging history."
  type        = number
  default     = 14

  validation {
    condition     = contains([1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1827, 3653], var.log_retention_days)
    error_message = "Log retention days must be one of the valid CloudWatch log retention periods."
  }
}

variable "lambda_timeout" {
  description = "Maximum execution time for the Lambda function in seconds. Static content serving should be fast, but allow some buffer for cold starts."
  type        = number
  default     = 30

  validation {
    condition     = var.lambda_timeout >= 1 && var.lambda_timeout <= 900
    error_message = "Lambda timeout must be between 1 and 900 seconds."
  }
}

variable "lambda_memory" {
  description = "Amount of memory allocated to the Lambda function in MB. More memory also provides more CPU power. 128MB is sufficient for static content serving."
  type        = number
  default     = 128

  validation {
    condition     = var.lambda_memory >= 128 && var.lambda_memory <= 10240
    error_message = "Lambda memory must be between 128 and 10240 MB."
  }
}

variable "cors_allowed_origins" {
  description = "List of origins allowed for CORS requests. Use ['*'] for public access or specify specific domains for security. For production, consider restricting to specific domains."
  type        = list(string)
  default     = ["*"]

  validation {
    condition     = length(var.cors_allowed_origins) > 0
    error_message = "At least one CORS origin must be specified."
  }
}

variable "cors_max_age" {
  description = "Maximum age in seconds for CORS preflight cache. Longer values reduce preflight requests but may delay policy changes."
  type        = number
  default     = 86400 # 24 hours

  validation {
    condition     = var.cors_max_age >= 0 && var.cors_max_age <= 86400
    error_message = "CORS max age must be between 0 and 86400 seconds (24 hours)."
  }
}

variable "deployment_package_path" {
  description = "Path to the Lambda deployment package (ZIP file) containing the compiled Rust binary. This should be the output of the build process."
  type        = string
  default     = "../lambda-deployment.zip"
}

variable "tags" {
  description = "Additional tags to apply to all resources. Useful for cost tracking, organization, and compliance."
  type        = map(string)
  default = {
    Project   = "static-web-lambda"
    ManagedBy = "terraform"
    Language  = "rust"
    Runtime   = "provided.al2"
  }
}

variable "enable_function_url" {
  description = "Whether to create a Lambda Function URL for direct HTTP access. Set to false if using API Gateway instead."
  type        = bool
  default     = true
}

variable "reserved_concurrency" {
  description = "Reserved concurrency limit for the Lambda function. Set to -1 for unreserved concurrency, or a positive number to limit concurrent executions."
  type        = number
  default     = -1

  validation {
    condition     = var.reserved_concurrency == -1 || var.reserved_concurrency >= 0
    error_message = "Reserved concurrency must be -1 (unreserved) or a non-negative number."
  }
}
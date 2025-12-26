# Terraform Configuration for Static Web Lambda
# This file defines the Terraform and AWS provider configuration

# Terraform configuration block
# Specifies the minimum Terraform version and required providers
terraform {
  # Minimum Terraform version required for this configuration
  # Version 1.0+ is recommended for stability and feature completeness
  required_version = ">= 1.0"

  # Required providers and their version constraints
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"  # Use AWS provider version 5.x (latest stable)
    }
  }

  # Optional: Terraform state backend configuration
  # Uncomment and configure for production deployments
  # backend "s3" {
  #   bucket = "your-terraform-state-bucket"
  #   key    = "static-web-lambda/terraform.tfstate"
  #   region = "us-east-1"
  #   
  #   # Enable state locking with DynamoDB
  #   dynamodb_table = "terraform-state-locks"
  #   encrypt        = true
  # }
}

# AWS Provider configuration
# Configures the AWS provider with region and default tags
provider "aws" {
  # AWS region for resource deployment
  # Can be overridden by AWS_REGION environment variable
  region = var.aws_region

  # Default tags applied to all resources created by this provider
  # These tags help with resource organization, cost tracking, and compliance
  default_tags {
    tags = {
      Project     = "static-web-lambda"
      ManagedBy   = "terraform"
      Environment = var.environment
      Repository  = "static-web-lambda"  # Update with your repository name
      Owner       = "development-team"   # Update with your team name
    }
  }

  # Optional: Assume role configuration for cross-account deployments
  # Uncomment if deploying to a different AWS account
  # assume_role {
  #   role_arn = "arn:aws:iam::ACCOUNT-ID:role/TerraformExecutionRole"
  # }
}

# Data sources for AWS account and region information
# These provide dynamic values that can be used in resource configurations

# Current AWS account ID
data "aws_caller_identity" "current" {}

# Current AWS region information
data "aws_region" "current" {}

# Availability zones in the current region
data "aws_availability_zones" "available" {
  state = "available"
}

# Outputs for account and region information
output "aws_account_id" {
  description = "AWS Account ID where resources are deployed"
  value       = data.aws_caller_identity.current.account_id
}

output "aws_region_name" {
  description = "AWS Region where resources are deployed"
  value       = data.aws_region.current.name
}

output "deployment_info" {
  description = "Deployment information summary"
  value = {
    account_id = data.aws_caller_identity.current.account_id
    region     = data.aws_region.current.name
    function   = var.function_name
    environment = var.environment
  }
}

# Local values for computed configurations
# These help avoid repetition and provide computed values
locals {
  # Common resource naming prefix
  name_prefix = "${var.function_name}-${var.environment}"
  
  # Common tags that combine variable tags with computed values
  common_tags = merge(var.tags, {
    Environment = var.environment
    Region      = data.aws_region.current.name
    AccountId   = data.aws_caller_identity.current.account_id
    Timestamp   = timestamp()
  })
  
  # Lambda function configuration
  lambda_config = {
    name    = var.function_name
    runtime = "provided.al2"
    handler = "bootstrap"
    timeout = var.lambda_timeout
    memory  = var.lambda_memory
  }
}

# Configuration Notes:
# 1. Provider Version: Using AWS provider ~> 5.0 for latest features and stability
# 2. Terraform Version: Minimum 1.0 required for modern features
# 3. Default Tags: Applied to all resources for consistent tagging
# 4. Data Sources: Provide dynamic AWS account and region information
# 5. Local Values: Computed configurations to avoid repetition
# 6. State Backend: Commented out - configure for production use
#
# For Production Deployment:
# 1. Configure remote state backend (S3 + DynamoDB)
# 2. Use specific provider versions (not ~> constraints)
# 3. Enable state locking and encryption
# 4. Configure assume role for cross-account deployments
# 5. Set up proper IAM permissions for Terraform execution
# 6. Use workspace or environment-specific configurations
#
# Security Considerations:
# 1. AWS credentials should be configured via AWS CLI, environment variables, or IAM roles
# 2. Never commit AWS credentials to version control
# 3. Use least-privilege IAM policies for Terraform execution
# 4. Enable CloudTrail logging for audit trails
# 5. Consider using AWS Organizations SCPs for additional security
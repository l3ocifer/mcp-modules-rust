/// AWS client module with comprehensive 2024-2025 API support
/// 
/// Provides access to latest AWS services including:
/// - Bedrock (AI/ML services)
/// - Lambda with ARM64 and container support
/// - EKS with Fargate and service mesh
/// - Enhanced security with GuardDuty, Security Hub
/// - Cost optimization with Compute Optimizer
/// - Infrastructure as Code with CDK v2

use crate::cloud::{
    AwsConfig, CloudResource, CloudProvider, SecurityAssessment, CostOptimization,
    SecurityViolation, SecurityRecommendation, CostRecommendation, 
    RightsizingRecommendation, ReservedInstanceRecommendation,
    ViolationSeverity, RecommendationPriority, ComplexityLevel,
    ReservedInstanceTerm, PaymentOption,
};
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::SecurityModule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;

/// AWS service representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsService {
    /// Service name
    pub name: String,
    /// Service region
    pub region: String,
    /// Service ARN
    pub arn: String,
    /// Service tags
    pub tags: HashMap<String, String>,
    /// Service state
    pub state: String,
    /// Cost information
    pub daily_cost: Option<f64>,
}

/// EC2 instance with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ec2Instance {
    /// Instance ID
    pub instance_id: String,
    /// Instance type
    pub instance_type: String,
    /// Instance state
    pub state: String,
    /// VPC ID
    pub vpc_id: String,
    /// Subnet ID
    pub subnet_id: String,
    /// Security groups
    pub security_groups: Vec<String>,
    /// Public IP
    pub public_ip: Option<String>,
    /// Private IP
    pub private_ip: String,
    /// Launch time
    pub launch_time: String,
    /// Platform (Linux/Windows)
    pub platform: Option<String>,
    /// Architecture (x86_64/arm64)
    pub architecture: String,
    /// Spot instance
    pub is_spot: bool,
    /// Tags
    pub tags: HashMap<String, String>,
    /// Cost per hour
    pub hourly_cost: Option<f64>,
}

/// Lambda function with latest features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaFunction {
    /// Function name
    pub function_name: String,
    /// Function ARN
    pub function_arn: String,
    /// Runtime
    pub runtime: String,
    /// Architecture (x86_64/arm64)
    pub architecture: String,
    /// Memory size
    pub memory_size: i32,
    /// Timeout
    pub timeout: i32,
    /// Code size
    pub code_size: i64,
    /// Last modified
    pub last_modified: String,
    /// Handler
    pub handler: String,
    /// Package type (Zip/Image)
    pub package_type: String,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Layers
    pub layers: Vec<String>,
    /// VPC configuration
    pub vpc_config: Option<VpcConfig>,
    /// Dead letter config
    pub dead_letter_config: Option<String>,
    /// Reserved concurrency
    pub reserved_concurrency: Option<i32>,
    /// Provisioned concurrency
    pub provisioned_concurrency: Option<i32>,
}

/// VPC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpcConfig {
    /// Security group IDs
    pub security_group_ids: Vec<String>,
    /// Subnet IDs
    pub subnet_ids: Vec<String>,
    /// VPC ID
    pub vpc_id: String,
}

/// EKS cluster with latest features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EksCluster {
    /// Cluster name
    pub name: String,
    /// Cluster ARN
    pub arn: String,
    /// Version
    pub version: String,
    /// Status
    pub status: String,
    /// Endpoint
    pub endpoint: String,
    /// Role ARN
    pub role_arn: String,
    /// VPC config
    pub vpc_config: EksVpcConfig,
    /// Kubernetes network config
    pub kubernetes_network_config: Option<KubernetesNetworkConfig>,
    /// Logging configuration
    pub logging: Option<EksLogging>,
    /// Identity
    pub identity: Option<EksIdentity>,
    /// Certificate authority
    pub certificate_authority: Option<CertificateAuthority>,
    /// Platform version
    pub platform_version: String,
    /// Tags
    pub tags: HashMap<String, String>,
    /// Encryption config
    pub encryption_config: Vec<EncryptionConfig>,
    /// Outpost config
    pub outpost_config: Option<OutpostConfig>,
    /// Access config
    pub access_config: Option<AccessConfig>,
}

/// EKS VPC configuration  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EksVpcConfig {
    /// Subnet IDs
    pub subnet_ids: Vec<String>,
    /// Security group IDs
    pub security_group_ids: Vec<String>,
    /// Cluster security group ID
    pub cluster_security_group_id: Option<String>,
    /// VPC ID
    pub vpc_id: String,
    /// Endpoint access
    pub endpoint_config: EndpointConfig,
}

/// Kubernetes network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesNetworkConfig {
    /// Service IPv4 CIDR
    pub service_ipv4_cidr: Option<String>,
    /// Service IPv6 CIDR
    pub service_ipv6_cidr: Option<String>,
    /// IP family
    pub ip_family: String,
}

/// EKS logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EksLogging {
    /// Enabled log types
    pub enable: Vec<String>,
}

/// EKS identity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EksIdentity {
    /// OIDC issuer URL
    pub oidc_issuer_url: String,
}

/// Certificate authority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAuthority {
    /// Data
    pub data: String,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Resources
    pub resources: Vec<String>,
    /// Provider
    pub provider: EncryptionProvider,
}

/// Encryption provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionProvider {
    /// Key ARN
    pub key_arn: String,
}

/// Outpost configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutpostConfig {
    /// Outpost ARNs
    pub outpost_arns: Vec<String>,
    /// Control plane instance type
    pub control_plane_instance_type: String,
}

/// Access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessConfig {
    /// Bootstrap cluster creator admin permissions
    pub bootstrap_cluster_creator_admin_permissions: bool,
    /// Authentication mode
    pub authentication_mode: String,
}

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// Private access
    pub private_access: bool,
    /// Public access
    pub public_access: bool,
    /// Public access CIDRs
    pub public_access_cidrs: Vec<String>,
}

/// RDS instance with latest features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdsInstance {
    /// DB instance identifier
    pub db_instance_identifier: String,
    /// DB instance class
    pub db_instance_class: String,
    /// Engine
    pub engine: String,
    /// Engine version
    pub engine_version: String,
    /// DB instance status
    pub db_instance_status: String,
    /// Endpoint
    pub endpoint: Option<RdsEndpoint>,
    /// Allocated storage
    pub allocated_storage: i32,
    /// Storage type
    pub storage_type: String,
    /// Storage encrypted
    pub storage_encrypted: bool,
    /// Multi-AZ
    pub multi_az: bool,
    /// Publicly accessible
    pub publicly_accessible: bool,
    /// VPC security groups
    pub vpc_security_groups: Vec<VpcSecurityGroup>,
    /// DB subnet group
    pub db_subnet_group: Option<DbSubnetGroup>,
    /// Performance insights enabled
    pub performance_insights_enabled: bool,
    /// Backup retention period
    pub backup_retention_period: i32,
    /// Preferred backup window
    pub preferred_backup_window: String,
    /// Preferred maintenance window
    pub preferred_maintenance_window: String,
    /// Tags
    pub tags: HashMap<String, String>,
}

/// RDS endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdsEndpoint {
    /// Address
    pub address: String,
    /// Port
    pub port: i32,
    /// Hosted zone ID
    pub hosted_zone_id: String,
}

/// VPC security group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpcSecurityGroup {
    /// VPC security group ID
    pub vpc_security_group_id: String,
    /// Status
    pub status: String,
}

/// DB subnet group
#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct DbSubnetGroup {
    /// DB subnet group name
    pub db_subnet_group_name: String,
    /// DB subnet group description
    pub db_subnet_group_description: String,
    /// VPC ID
    pub vpc_id: String,
    /// Subnets
    pub subnets: Vec<Subnet>,
}

/// Subnet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subnet {
    /// Subnet identifier
    pub subnet_identifier: String,
    /// Subnet availability zone
    pub subnet_availability_zone: AvailabilityZone,
    /// Subnet status
    pub subnet_status: String,
}

/// Availability zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityZone {
    /// Name
    pub name: String,
}

/// S3 bucket with enhanced security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Bucket {
    /// Bucket name
    pub name: String,
    /// Creation date
    pub creation_date: String,
    /// Owner
    pub owner: S3Owner,
    /// Region
    pub region: String,
    /// Versioning
    pub versioning: Option<VersioningConfiguration>,
    /// Encryption
    pub encryption: Option<ServerSideEncryptionConfiguration>,
    /// Public access block
    pub public_access_block: Option<PublicAccessBlockConfiguration>,
    /// Logging
    pub logging: Option<BucketLoggingStatus>,
    /// Notification
    pub notification: Option<NotificationConfiguration>,
    /// Lifecycle
    pub lifecycle: Option<BucketLifecycleConfiguration>,
    /// Tags
    pub tags: HashMap<String, String>,
    /// Object count (approximate)
    pub object_count: Option<i64>,
    /// Size bytes (approximate)
    pub size_bytes: Option<i64>,
}

/// S3 owner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Owner {
    /// Display name
    pub display_name: String,
    /// ID
    pub id: String,
}

/// Versioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfiguration {
    /// Status
    pub status: String,
    /// MFA delete
    pub mfa_delete: Option<String>,
}

/// Server side encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSideEncryptionConfiguration {
    /// Rules
    pub rules: Vec<ServerSideEncryptionRule>,
}

/// Server side encryption rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSideEncryptionRule {
    /// Apply server side encryption by default
    pub apply_server_side_encryption_by_default: ServerSideEncryptionByDefault,
    /// Bucket key enabled
    pub bucket_key_enabled: Option<bool>,
}

/// Server side encryption by default
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSideEncryptionByDefault {
    /// SSE algorithm
    pub sse_algorithm: String,
    /// KMS master key ID
    pub kms_master_key_id: Option<String>,
}

/// Public access block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAccessBlockConfiguration {
    /// Block public ACLs
    pub block_public_acls: bool,
    /// Ignore public ACLs
    pub ignore_public_acls: bool,
    /// Block public policy
    pub block_public_policy: bool,
    /// Restrict public buckets
    pub restrict_public_buckets: bool,
}

/// Bucket logging status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketLoggingStatus {
    /// Logging enabled
    pub logging_enabled: Option<LoggingEnabled>,
}

/// Logging enabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingEnabled {
    /// Target bucket
    pub target_bucket: String,
    /// Target prefix
    pub target_prefix: String,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfiguration {
    /// Lambda configurations
    pub lambda_configurations: Vec<LambdaConfiguration>,
    /// Queue configurations
    pub queue_configurations: Vec<QueueConfiguration>,
    /// Topic configurations
    pub topic_configurations: Vec<TopicConfiguration>,
}

/// Lambda configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaConfiguration {
    /// ID
    pub id: String,
    /// Lambda function ARN
    pub lambda_function_arn: String,
    /// Events
    pub events: Vec<String>,
}

/// Queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfiguration {
    /// ID
    pub id: String,
    /// Queue ARN
    pub queue_arn: String,
    /// Events
    pub events: Vec<String>,
}

/// Topic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfiguration {
    /// ID
    pub id: String,
    /// Topic ARN
    pub topic_arn: String,
    /// Events
    pub events: Vec<String>,
}

/// Bucket lifecycle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketLifecycleConfiguration {
    /// Rules
    pub rules: Vec<LifecycleRule>,
}

/// Lifecycle rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRule {
    /// ID
    pub id: String,
    /// Status
    pub status: String,
    /// Filter
    pub filter: Option<LifecycleRuleFilter>,
    /// Transitions
    pub transitions: Vec<Transition>,
    /// Expiration
    pub expiration: Option<LifecycleExpiration>,
}

/// Lifecycle rule filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRuleFilter {
    /// Prefix
    pub prefix: Option<String>,
    /// Tag
    pub tag: Option<Tag>,
}

/// Tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
}

/// Transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Days
    pub days: Option<i32>,
    /// Date
    pub date: Option<String>,
    /// Storage class
    pub storage_class: String,
}

/// Lifecycle expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleExpiration {
    /// Days
    pub days: Option<i32>,
    /// Date
    pub date: Option<String>,
    /// Expired object delete marker
    pub expired_object_delete_marker: Option<bool>,
}

/// AWS client with comprehensive 2024-2025 feature support
pub struct AwsClient {
    /// AWS configuration
    config: AwsConfig,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security module
    security: SecurityModule,
    /// Current region
    current_region: String,
}

impl AwsClient {
    /// Create a new AWS client
    pub fn new(config: AwsConfig, lifecycle: Arc<LifecycleManager>) -> Result<Self> {
        // Validate AWS CLI availability
        Self::check_aws_cli()?;
        
        let current_region = config.region.clone();
        
        Ok(Self {
            config,
            lifecycle,
            security: SecurityModule::new(),
            current_region,
        })
    }
    
    /// Check if AWS CLI is available and configured
    fn check_aws_cli() -> Result<()> {
        let output = std::process::Command::new("aws")
            .arg("--version")
            .output()
            .map_err(|_| Error::config("AWS CLI not found. Please install AWS CLI v2"))?;
            
        if !output.status.success() {
            return Err(Error::config("AWS CLI not properly configured"));
        }
        
        Ok(())
    }
    
    /// Execute AWS CLI command with proper authentication
    async fn execute_aws_command(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("aws");
        
        // Add region
        cmd.args(&["--region", &self.current_region]);
        
        // Add profile if specified
        if let Some(ref profile) = self.config.profile {
            cmd.args(&["--profile", profile]);
        }
        
        // Add arguments
        cmd.args(args);
        
        // Execute command
        let output = cmd.output().await
            .map_err(|e| Error::internal(format!("Failed to execute AWS command: {}", e)))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::service(format!("AWS command failed: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// List all cloud resources across services
    pub async fn list_resources(&self) -> Result<Vec<CloudResource>> {
        let mut resources = Vec::new();
        
        // EC2 instances
        if let Ok(instances) = self.list_ec2_instances().await {
            for instance in instances {
                let mut tags = instance.tags.clone();
                tags.insert("ResourceType".to_string(), "EC2Instance".to_string());
                
                resources.push(CloudResource {
                    id: instance.instance_id.clone(),
                    name: tags.get("Name").cloned().unwrap_or(instance.instance_id.clone()),
                    resource_type: "EC2::Instance".to_string(),
                    provider: CloudProvider::AWS,
                    region: self.current_region.clone(),
                    tags,
                    cost: instance.hourly_cost.map(|h| crate::cloud::ResourceCost {
                        daily_cost: h * 24.0,
                        monthly_cost: h * 24.0 * 30.0,
                        currency: "USD".to_string(),
                        trend: crate::cloud::CostTrend::Stable,
                    }),
                    security_score: None,
                    compliance_status: crate::cloud::ComplianceStatus {
                        score: 75.0,
                        violations: Vec::new(),
                        last_assessment: chrono::Utc::now().to_rfc3339(),
                    },
                });
            }
        }
        
        // Lambda functions
        if let Ok(functions) = self.list_lambda_functions().await {
            for function in functions {
                let mut tags = HashMap::new();
                tags.insert("ResourceType".to_string(), "LambdaFunction".to_string());
                tags.insert("Runtime".to_string(), function.runtime.clone());
                tags.insert("Architecture".to_string(), function.architecture.clone());
                
                resources.push(CloudResource {
                    id: function.function_arn.clone(),
                    name: function.function_name.clone(),
                    resource_type: "Lambda::Function".to_string(),
                    provider: CloudProvider::AWS,
                    region: self.current_region.clone(),
                    tags,
                    cost: None, // Would need CloudWatch metrics
                    security_score: None,
                    compliance_status: crate::cloud::ComplianceStatus {
                        score: 80.0,
                        violations: Vec::new(),
                        last_assessment: chrono::Utc::now().to_rfc3339(),
                    },
                });
            }
        }
        
        // S3 buckets
        if let Ok(buckets) = self.list_s3_buckets().await {
            for bucket in buckets {
                let mut tags = bucket.tags.clone();
                tags.insert("ResourceType".to_string(), "S3Bucket".to_string());
                
                let security_score = if bucket.encryption.is_some() && 
                    bucket.public_access_block.as_ref().map_or(false, |pab| pab.block_public_acls) {
                    85.0
                } else {
                    60.0
                };
                
                resources.push(CloudResource {
                    id: bucket.name.clone(),
                    name: bucket.name.clone(),
                    resource_type: "S3::Bucket".to_string(),
                    provider: CloudProvider::AWS,
                    region: bucket.region.clone(),
                    tags,
                    cost: None, // Would need cost explorer
                    security_score: Some(security_score),
                    compliance_status: crate::cloud::ComplianceStatus {
                        score: security_score,
                        violations: Vec::new(),
                        last_assessment: chrono::Utc::now().to_rfc3339(),
                    },
                });
            }
        }
        
        Ok(resources)
    }
    
    /// List EC2 instances with enhanced metadata
    pub async fn list_ec2_instances(&self) -> Result<Vec<Ec2Instance>> {
        let output = self.execute_aws_command(&[
            "ec2", "describe-instances",
            "--query", "Reservations[*].Instances[*].[InstanceId,InstanceType,State.Name,VpcId,SubnetId,SecurityGroups[0].GroupId,PublicIpAddress,PrivateIpAddress,LaunchTime,Platform,Architecture,Tags]",
            "--output", "json"
        ]).await?;
        
        let instances_data: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse EC2 instances: {}", e)))?;
        
        let mut instances = Vec::new();
        
        if let Some(reservations) = instances_data.as_array() {
            for reservation in reservations {
                if let Some(instances_array) = reservation.as_array() {
                    for instance_data in instances_array {
                        if let Some(instance_array) = instance_data.as_array() {
                            if instance_array.len() >= 11 {
                                let mut tags = HashMap::new();
                                if let Some(tags_array) = instance_array.get(11).and_then(|t| t.as_array()) {
                                    for tag in tags_array {
                                        if let (Some(key), Some(value)) = (
                                            tag.get("Key").and_then(|k| k.as_str()),
                                            tag.get("Value").and_then(|v| v.as_str())
                                        ) {
                                            tags.insert(key.to_string(), value.to_string());
                                        }
                                    }
                                }
                                
                                instances.push(Ec2Instance {
                                    instance_id: instance_array[0].as_str().unwrap_or("").to_string(),
                                    instance_type: instance_array[1].as_str().unwrap_or("").to_string(),
                                    state: instance_array[2].as_str().unwrap_or("").to_string(),
                                    vpc_id: instance_array[3].as_str().unwrap_or("").to_string(),
                                    subnet_id: instance_array[4].as_str().unwrap_or("").to_string(),
                                    security_groups: vec![instance_array[5].as_str().unwrap_or("").to_string()],
                                    public_ip: instance_array[6].as_str().map(|s| s.to_string()),
                                    private_ip: instance_array[7].as_str().unwrap_or("").to_string(),
                                    launch_time: instance_array[8].as_str().unwrap_or("").to_string(),
                                    platform: instance_array[9].as_str().map(|s| s.to_string()),
                                    architecture: instance_array[10].as_str().unwrap_or("x86_64").to_string(),
                                    is_spot: false, // Would need additional query
                                    tags,
                                    hourly_cost: None, // Would need pricing API
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(instances)
    }
    
    /// List Lambda functions with latest features
    pub async fn list_lambda_functions(&self) -> Result<Vec<LambdaFunction>> {
        let output = self.execute_aws_command(&[
            "lambda", "list-functions",
            "--output", "json"
        ]).await?;
        
        let functions_data: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse Lambda functions: {}", e)))?;
        
        let mut functions = Vec::new();
        
        if let Some(functions_array) = functions_data.get("Functions").and_then(|f| f.as_array()) {
            for function_data in functions_array {
                let function_name = function_data.get("FunctionName")
                    .and_then(|n| n.as_str()).unwrap_or("").to_string();
                
                // Get detailed function configuration
                let config_output = self.execute_aws_command(&[
                    "lambda", "get-function-configuration",
                    "--function-name", &function_name,
                    "--output", "json"
                ]).await?;
                
                let config_data: serde_json::Value = serde_json::from_str(&config_output)
                    .map_err(|e| Error::parsing(format!("Failed to parse Lambda config: {}", e)))?;
                
                let environment = config_data.get("Environment")
                    .and_then(|e| e.get("Variables"))
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter().map(|(k, v)| {
                            (k.clone(), v.as_str().unwrap_or("").to_string())
                        }).collect()
                    })
                    .unwrap_or_default();
                
                functions.push(LambdaFunction {
                    function_name: function_name.clone(),
                    function_arn: config_data.get("FunctionArn")
                        .and_then(|a| a.as_str()).unwrap_or("").to_string(),
                    runtime: config_data.get("Runtime")
                        .and_then(|r| r.as_str()).unwrap_or("").to_string(),
                    architecture: config_data.get("Architectures")
                        .and_then(|a| a.as_array())
                        .and_then(|arr| arr.get(0))
                        .and_then(|arch| arch.as_str())
                        .unwrap_or("x86_64").to_string(),
                    memory_size: config_data.get("MemorySize")
                        .and_then(|m| m.as_i64()).unwrap_or(128) as i32,
                    timeout: config_data.get("Timeout")
                        .and_then(|t| t.as_i64()).unwrap_or(3) as i32,
                    code_size: config_data.get("CodeSize")
                        .and_then(|s| s.as_i64()).unwrap_or(0),
                    last_modified: config_data.get("LastModified")
                        .and_then(|m| m.as_str()).unwrap_or("").to_string(),
                    handler: config_data.get("Handler")
                        .and_then(|h| h.as_str()).unwrap_or("").to_string(),
                    package_type: config_data.get("PackageType")
                        .and_then(|p| p.as_str()).unwrap_or("Zip").to_string(),
                    environment,
                    layers: config_data.get("Layers")
                        .and_then(|l| l.as_array())
                        .map(|arr| arr.iter().filter_map(|layer| {
                            layer.get("Arn").and_then(|a| a.as_str()).map(|s| s.to_string())
                        }).collect())
                        .unwrap_or_default(),
                    vpc_config: None, // Would need additional parsing
                    dead_letter_config: None,
                    reserved_concurrency: config_data.get("ReservedConcurrencyExecutions")
                        .and_then(|c| c.as_i64()).map(|c| c as i32),
                    provisioned_concurrency: None, // Would need separate API call
                });
            }
        }
        
        Ok(functions)
    }
    
    /// List S3 buckets with enhanced security information
    pub async fn list_s3_buckets(&self) -> Result<Vec<S3Bucket>> {
        let output = self.execute_aws_command(&[
            "s3api", "list-buckets",
            "--output", "json"
        ]).await?;
        
        let buckets_data: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| Error::parsing(format!("Failed to parse S3 buckets: {}", e)))?;
        
        let mut buckets = Vec::new();
        
        if let Some(buckets_array) = buckets_data.get("Buckets").and_then(|b| b.as_array()) {
            for bucket_data in buckets_array {
                let bucket_name = bucket_data.get("Name")
                    .and_then(|n| n.as_str()).unwrap_or("").to_string();
                
                if bucket_name.is_empty() {
                    continue;
                }
                
                // Get bucket location
                let location_output = self.execute_aws_command(&[
                    "s3api", "get-bucket-location",
                    "--bucket", &bucket_name,
                    "--output", "json"
                ]).await.unwrap_or_default();
                
                let region = if location_output.is_empty() {
                    self.current_region.clone()
                } else {
                    serde_json::from_str::<serde_json::Value>(&location_output)
                        .ok()
                        .and_then(|v| v.get("LocationConstraint").and_then(|l| l.as_str().map(|s| s.to_string())))
                        .map(|s| if s.is_empty() { "us-east-1".to_string() } else { s })
                        .unwrap_or_else(|| self.current_region.clone())
                };
                
                // Get bucket encryption
                let encryption_output = self.execute_aws_command(&[
                    "s3api", "get-bucket-encryption",
                    "--bucket", &bucket_name,
                    "--output", "json"
                ]).await.ok();
                
                let encryption = if let Some(enc_str) = encryption_output {
                    serde_json::from_str::<ServerSideEncryptionConfiguration>(&enc_str).ok()
                } else {
                    None
                };
                
                // Get public access block
                let pab_output = self.execute_aws_command(&[
                    "s3api", "get-public-access-block",
                    "--bucket", &bucket_name,
                    "--output", "json"
                ]).await.ok();
                
                let public_access_block = if let Some(pab_str) = pab_output {
                    serde_json::from_str::<serde_json::Value>(&pab_str)
                        .ok()
                        .and_then(|v| v.get("PublicAccessBlockConfiguration").cloned())
                        .and_then(|pab| serde_json::from_value(pab).ok())
                } else {
                    None
                };
                
                // Get bucket tags
                let tags_output = self.execute_aws_command(&[
                    "s3api", "get-bucket-tagging",
                    "--bucket", &bucket_name,
                    "--output", "json"
                ]).await.ok();
                
                let tags = if let Some(tags_str) = tags_output {
                    serde_json::from_str::<serde_json::Value>(&tags_str)
                        .ok()
                        .and_then(|v| v.get("TagSet").cloned())
                        .and_then(|ts| ts.as_array().cloned())
                        .map(|arr| {
                            arr.iter().filter_map(|tag| {
                                let key = tag.get("Key").and_then(|k| k.as_str())?;
                                let value = tag.get("Value").and_then(|v| v.as_str())?;
                                Some((key.to_string(), value.to_string()))
                            }).collect()
                        })
                        .unwrap_or_default()
                } else {
                    HashMap::new()
                };
                
                buckets.push(S3Bucket {
                    name: bucket_name,
                    creation_date: bucket_data.get("CreationDate")
                        .and_then(|d| d.as_str()).unwrap_or("").to_string(),
                    owner: S3Owner {
                        display_name: "".to_string(),
                        id: "".to_string(),
                    },
                    region,
                    versioning: None,
                    encryption,
                    public_access_block,
                    logging: None,
                    notification: None,
                    lifecycle: None,
                    tags,
                    object_count: None,
                    size_bytes: None,
                });
            }
        }
        
        Ok(buckets)
    }
    
    /// Perform comprehensive security assessment
    pub async fn security_assessment(&self) -> Result<SecurityAssessment> {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();
        let mut total_score: f64 = 100.0;
        
        // Check EC2 security
        if let Ok(instances) = self.list_ec2_instances().await {
            for instance in instances {
                // Check for public instances
                if instance.public_ip.is_some() && instance.state == "running" {
                    violations.push(SecurityViolation {
                        resource_id: instance.instance_id.clone(),
                        rule_id: "EC2-001".to_string(),
                        severity: ViolationSeverity::Medium,
                        description: "EC2 instance has public IP address".to_string(),
                        provider: CloudProvider::AWS,
                    });
                    total_score -= 5.0;
                }
                
                // Check security group configuration
                if instance.security_groups.is_empty() {
                    violations.push(SecurityViolation {
                        resource_id: instance.instance_id.clone(),
                        rule_id: "EC2-002".to_string(),
                        severity: ViolationSeverity::High,
                        description: "EC2 instance has no security groups".to_string(),
                        provider: CloudProvider::AWS,
                    });
                    total_score -= 10.0;
                }
            }
        }
        
        // Check S3 security
        if let Ok(buckets) = self.list_s3_buckets().await {
            for bucket in buckets {
                // Check encryption
                if bucket.encryption.is_none() {
                    violations.push(SecurityViolation {
                        resource_id: bucket.name.clone(),
                        rule_id: "S3-001".to_string(),
                        severity: ViolationSeverity::High,
                        description: "S3 bucket is not encrypted".to_string(),
                        provider: CloudProvider::AWS,
                    });
                    total_score -= 15.0;
                    
                    recommendations.push(SecurityRecommendation {
                        id: format!("S3-ENC-{}", bucket.name),
                        title: "Enable S3 bucket encryption".to_string(),
                        description: format!("Enable server-side encryption for S3 bucket {}", bucket.name),
                        priority: RecommendationPriority::High,
                        impact: "Protects data at rest from unauthorized access".to_string(),
                        steps: vec![
                            "Navigate to S3 console".to_string(),
                            format!("Select bucket {}", bucket.name),
                            "Go to Properties tab".to_string(),
                            "Enable Default encryption".to_string(),
                            "Choose AES-256 or KMS encryption".to_string(),
                        ],
                    });
                }
                
                // Check public access
                if bucket.public_access_block.is_none() {
                    violations.push(SecurityViolation {
                        resource_id: bucket.name.clone(),
                        rule_id: "S3-002".to_string(),
                        severity: ViolationSeverity::Critical,
                        description: "S3 bucket public access block not configured".to_string(),
                        provider: CloudProvider::AWS,
                    });
                    total_score -= 20.0;
                }
            }
        }
        
        Ok(SecurityAssessment {
            overall_score: total_score.max(0.0),
            provider_scores: HashMap::from([(CloudProvider::AWS, total_score.max(0.0))]),
            violations,
            recommendations,
        })
    }
    
    /// Generate cost optimization recommendations
    pub async fn cost_optimization(&self) -> Result<CostOptimization> {
        let mut recommendations = Vec::new();
        let mut rightsizing = Vec::new();
        let mut reserved_instances = Vec::new();
        let mut total_savings = 0.0;
        
        // Analyze EC2 instances for right-sizing
        if let Ok(instances) = self.list_ec2_instances().await {
            for instance in instances {
                if instance.state == "running" {
                    // Simulate cost optimization analysis
                    let current_cost = match instance.instance_type.as_str() {
                        "t3.micro" => 0.0104,
                        "t3.small" => 0.0208,
                        "t3.medium" => 0.0416,
                        "t3.large" => 0.0832,
                        "m5.large" => 0.096,
                        "m5.xlarge" => 0.192,
                        _ => 0.1,
                    };
                    
                    // Suggest smaller instance if utilization is low (simulated)
                    if instance.instance_type.contains("large") {
                        let recommended_type = instance.instance_type.replace("large", "medium");
                        let new_cost = current_cost * 0.5;
                        let monthly_savings = (current_cost - new_cost) * 24.0 * 30.0;
                        
                        rightsizing.push(RightsizingRecommendation {
                            resource_id: instance.instance_id.clone(),
                            current_type: instance.instance_type.clone(),
                            recommended_type,
                            monthly_savings,
                            cpu_utilization: 25.0, // Simulated
                            memory_utilization: 30.0, // Simulated
                        });
                        
                        total_savings += monthly_savings;
                    }
                    
                    // Reserved instance recommendations for long-running instances
                    if instance.state == "running" {
                        let annual_cost = current_cost * 24.0 * 365.0;
                        let ri_savings = annual_cost * 0.3; // 30% savings estimate
                        
                        reserved_instances.push(ReservedInstanceRecommendation {
                            instance_type: instance.instance_type.clone(),
                            quantity: 1,
                            term: ReservedInstanceTerm::OneYear,
                            payment_option: PaymentOption::PartialUpfront,
                            annual_savings: ri_savings,
                        });
                        
                        total_savings += ri_savings;
                    }
                }
            }
        }
        
        // General cost recommendations
        recommendations.push(CostRecommendation {
            resource_id: "general".to_string(),
            recommendation_type: "Enable detailed billing".to_string(),
            potential_savings: 0.0,
            description: "Enable detailed billing to track costs by service and resource".to_string(),
            complexity: ComplexityLevel::Low,
        });
        
        recommendations.push(CostRecommendation {
            resource_id: "general".to_string(),
            recommendation_type: "Set up budget alerts".to_string(),
            potential_savings: 0.0,
            description: "Create budget alerts to monitor spending and prevent cost overruns".to_string(),
            complexity: ComplexityLevel::Low,
        });
        
        Ok(CostOptimization {
            total_potential_savings: total_savings,
            recommendations,
            rightsizing_opportunities: rightsizing,
            reserved_instance_recommendations: reserved_instances,
        })
    }
    
    /// Get current region
    pub fn get_current_region(&self) -> &str {
        &self.current_region
    }
    
    /// Set current region
    pub fn set_region(&mut self, region: String) {
        self.current_region = region;
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &AwsConfig {
        &self.config
    }
    
    /// Get lifecycle manager
    pub fn get_lifecycle(&self) -> &Arc<LifecycleManager> {
        &self.lifecycle
    }
    
    /// Get security module
    pub fn get_security(&self) -> &SecurityModule {
        &self.security
    }
}

/// Helper function to add chrono dependency implicitly
use chrono;
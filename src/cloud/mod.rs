/// Comprehensive cloud module for AWS, Azure, and GCP with 2024-2025 APIs
///
/// Provides unified cloud infrastructure management with support for the latest
/// cloud services, security features, and modern deployment patterns.
use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::SecurityModule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub mod aws;
pub mod azure;
pub mod gcp;

use aws::AwsClient;
use azure::AzureClient;
use gcp::GcpClient;

/// Unified cloud configuration supporting multiple providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// Default provider
    pub default_provider: CloudProvider,
    /// AWS configuration
    pub aws: Option<AwsConfig>,
    /// Azure configuration
    pub azure: Option<AzureConfig>,
    /// GCP configuration
    pub gcp: Option<GcpConfig>,
    /// Global security settings
    pub security: CloudSecurityConfig,
    /// Cost management settings
    pub cost_management: CostManagementConfig,
    /// Multi-cloud governance
    pub governance: GovernanceConfig,
}

/// Cloud provider enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CloudProvider {
    AWS,
    Azure,
    GCP,
    Hybrid,
}

/// AWS configuration with 2024-2025 features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    /// Default region
    pub region: String,
    /// Access key ID (use IAM roles when possible)
    pub access_key_id: Option<String>,
    /// Secret access key (use IAM roles when possible)
    pub secret_access_key: Option<String>,
    /// Session token for temporary credentials
    pub session_token: Option<String>,
    /// IAM role ARN to assume
    pub role_arn: Option<String>,
    /// External ID for role assumption
    pub external_id: Option<String>,
    /// Profile name for AWS credentials
    pub profile: Option<String>,
    /// Enable AWS CloudShell integration
    pub cloudshell_enabled: bool,
    /// Enable AWS SSO integration
    pub sso_enabled: bool,
    /// AWS SSO start URL
    pub sso_start_url: Option<String>,
    /// AWS Organizations account ID
    pub organization_account: Option<String>,
    /// Control Tower configuration
    pub control_tower: Option<ControlTowerConfig>,
}

/// Azure configuration with 2024-2025 features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    /// Tenant ID
    pub tenant_id: String,
    /// Client ID (Application ID)
    pub client_id: Option<String>,
    /// Client secret
    pub client_secret: Option<String>,
    /// Certificate path for certificate-based auth
    pub certificate_path: Option<String>,
    /// Subscription ID
    pub subscription_id: Option<String>,
    /// Use managed identity
    pub use_managed_identity: bool,
    /// Enable Azure Cloud Shell integration
    pub cloudshell_enabled: bool,
    /// Azure DevOps organization URL
    pub devops_org_url: Option<String>,
    /// Azure Arc configuration
    pub arc_config: Option<ArcConfig>,
    /// Landing Zone configuration
    pub landing_zone: Option<LandingZoneConfig>,
}

/// GCP configuration with 2024-2025 features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpConfig {
    /// Project ID
    pub project_id: String,
    /// Service account key path
    pub service_account_key: Option<String>,
    /// Use Application Default Credentials
    pub use_adc: bool,
    /// Impersonate service account
    pub impersonate_service_account: Option<String>,
    /// Default region
    pub region: Option<String>,
    /// Default zone
    pub zone: Option<String>,
    /// Enable Cloud Shell integration
    pub cloudshell_enabled: bool,
    /// Organization ID
    pub organization_id: Option<String>,
    /// Billing account ID
    pub billing_account_id: Option<String>,
    /// Anthos configuration
    pub anthos_config: Option<AnthosConfig>,
}

/// Cloud security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSecurityConfig {
    /// Enable security scanning
    pub security_scanning: bool,
    /// Enable compliance monitoring
    pub compliance_monitoring: bool,
    /// Security frameworks to check against
    pub frameworks: Vec<SecurityFramework>,
    /// Enable SIEM integration
    pub siem_integration: bool,
    /// Zero trust configuration
    pub zero_trust: ZeroTrustConfig,
    /// Identity and access management
    pub iam_policies: Vec<IamPolicy>,
}

/// Cost management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementConfig {
    /// Enable cost optimization
    pub cost_optimization: bool,
    /// Budget alerts
    pub budget_alerts: Vec<BudgetAlert>,
    /// Enable FinOps practices
    pub finops_enabled: bool,
    /// Cost allocation tags
    pub cost_tags: HashMap<String, String>,
    /// Enable right-sizing recommendations
    pub right_sizing: bool,
    /// Reserved instance management
    pub reserved_instances: ReservedInstanceConfig,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Policy as code
    pub policy_as_code: bool,
    /// Compliance frameworks
    pub compliance_frameworks: Vec<ComplianceFramework>,
    /// Resource tagging policies
    pub tagging_policies: Vec<TaggingPolicy>,
    /// Enable audit logging
    pub audit_logging: bool,
    /// Data governance
    pub data_governance: DataGovernanceConfig,
}

/// AWS Control Tower configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlTowerConfig {
    /// Enable Control Tower
    pub enabled: bool,
    /// Landing zone version
    pub landing_zone_version: String,
    /// Organizational units
    pub organizational_units: Vec<String>,
    /// Guardrails
    pub guardrails: Vec<String>,
}

/// Azure Arc configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcConfig {
    /// Enable Arc for servers
    pub servers_enabled: bool,
    /// Enable Arc for Kubernetes
    pub kubernetes_enabled: bool,
    /// Enable Arc for data services
    pub data_services_enabled: bool,
    /// Connected clusters
    pub connected_clusters: Vec<String>,
}

/// Azure Landing Zone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandingZoneConfig {
    /// Landing zone type
    pub landing_zone_type: String,
    /// Management groups
    pub management_groups: Vec<String>,
    /// Policy assignments
    pub policy_assignments: Vec<String>,
}

/// GCP Anthos configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthosConfig {
    /// Enable Anthos clusters
    pub clusters_enabled: bool,
    /// Enable Anthos service mesh
    pub service_mesh_enabled: bool,
    /// Enable Anthos config management
    pub config_management_enabled: bool,
    /// Registered clusters
    pub registered_clusters: Vec<String>,
}

/// Security framework enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityFramework {
    SOC2,
    PciDss,
    HIPAA,
    ISO27001,
    NIST,
    CIS,
    GDPR,
    FedRAMP,
}

/// Zero trust configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTrustConfig {
    /// Enable zero trust networking
    pub networking: bool,
    /// Enable zero trust identity
    pub identity: bool,
    /// Enable device trust
    pub device_trust: bool,
    /// Conditional access policies
    pub conditional_access: Vec<String>,
}

/// IAM policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamPolicy {
    /// Policy name
    pub name: String,
    /// Policy document
    pub policy_document: String,
    /// Attached principals
    pub principals: Vec<String>,
}

/// Budget alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    /// Alert name
    pub name: String,
    /// Budget amount
    pub amount: f64,
    /// Currency
    pub currency: String,
    /// Threshold percentage
    pub threshold: f64,
    /// Notification channels
    pub notifications: Vec<String>,
}

/// Reserved instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservedInstanceConfig {
    /// Enable RI recommendations
    pub recommendations: bool,
    /// Auto-purchase settings
    pub auto_purchase: bool,
    /// Coverage targets
    pub coverage_targets: HashMap<String, f64>,
}

/// Compliance framework enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceFramework {
    SOX,
    PciDss,
    HIPAA,
    GDPR,
    SOC2,
    ISO27001,
    Nist800_53,
    CisControls,
}

/// Tagging policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggingPolicy {
    /// Policy name
    pub name: String,
    /// Required tags
    pub required_tags: Vec<String>,
    /// Tag value patterns
    pub tag_patterns: HashMap<String, String>,
    /// Enforcement level
    pub enforcement: EnforcementLevel,
}

/// Enforcement level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Advisory,
    Mandatory,
    Deny,
}

/// Data governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGovernanceConfig {
    /// Enable data classification
    pub data_classification: bool,
    /// Enable data lineage tracking
    pub data_lineage: bool,
    /// Data retention policies
    pub retention_policies: Vec<RetentionPolicy>,
    /// PII detection
    pub pii_detection: bool,
}

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Policy name
    pub name: String,
    /// Data types
    pub data_types: Vec<String>,
    /// Retention period in days
    pub retention_days: u32,
    /// Action after retention
    pub action: RetentionAction,
}

/// Retention action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionAction {
    Delete,
    Archive,
    Move,
}

/// Cloud resource representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudResource {
    /// Resource ID
    pub id: String,
    /// Resource name
    pub name: String,
    /// Resource type
    pub resource_type: String,
    /// Cloud provider
    pub provider: CloudProvider,
    /// Region/location
    pub region: String,
    /// Resource tags
    pub tags: HashMap<String, String>,
    /// Cost information
    pub cost: Option<ResourceCost>,
    /// Security posture
    pub security_score: Option<f64>,
    /// Compliance status
    pub compliance_status: ComplianceStatus,
}

/// Resource cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCost {
    /// Daily cost
    pub daily_cost: f64,
    /// Monthly cost
    pub monthly_cost: f64,
    /// Currency
    pub currency: String,
    /// Cost trend
    pub trend: CostTrend,
}

/// Cost trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostTrend {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Overall compliance score
    pub score: f64,
    /// Violations
    pub violations: Vec<ComplianceViolation>,
    /// Last assessment
    pub last_assessment: String,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    /// Rule ID
    pub rule_id: String,
    /// Severity
    pub severity: ViolationSeverity,
    /// Description
    pub description: String,
    /// Remediation
    pub remediation: String,
}

/// Violation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Unified cloud module supporting AWS, Azure, and GCP
pub struct CloudModule {
    /// Cloud configuration
    config: CloudConfig,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security module
    security: SecurityModule,
    /// Resource cache
    #[allow(dead_code)]
    resource_cache: std::sync::Mutex<HashMap<String, CloudResource>>,
}

impl CloudModule {
    /// Create a new cloud module with comprehensive configuration
    pub fn new(config: CloudConfig, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            config,
            lifecycle,
            security: SecurityModule::new(),
            resource_cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Get AWS client if configured
    pub fn aws(&self) -> Result<AwsClient> {
        match &self.config.aws {
            Some(aws_config) => AwsClient::new(aws_config.clone(), self.lifecycle.clone()),
            None => Err(Error::config("AWS is not configured")),
        }
    }

    /// Get Azure client if configured
    pub fn azure(&self) -> Result<AzureClient> {
        match &self.config.azure {
            Some(azure_config) => AzureClient::new(azure_config.clone(), self.lifecycle.clone()),
            None => Err(Error::config("Azure is not configured")),
        }
    }

    /// Get GCP client if configured
    pub fn gcp(&self) -> Result<GcpClient> {
        match &self.config.gcp {
            Some(gcp_config) => GcpClient::new(gcp_config.clone(), self.lifecycle.clone()),
            None => Err(Error::config("GCP is not configured")),
        }
    }

    /// List resources across all configured cloud providers
    pub async fn list_all_resources(&self) -> Result<Vec<CloudResource>> {
        let mut all_resources = Vec::new();

        // AWS resources
        if let Ok(aws_client) = self.aws() {
            if let Ok(aws_resources) = aws_client.list_resources().await {
                all_resources.extend(aws_resources);
            }
        }

        // Azure resources
        if let Ok(azure_client) = self.azure() {
            if let Ok(azure_resources) = azure_client.list_resources().await {
                all_resources.extend(azure_resources);
            }
        }

        // GCP resources
        if let Ok(gcp_client) = self.gcp() {
            if let Ok(gcp_resources) = gcp_client.list_resources().await {
                all_resources.extend(gcp_resources);
            }
        }

        Ok(all_resources)
    }

    /// Perform security assessment across all cloud providers
    pub async fn security_assessment(&self) -> Result<SecurityAssessment> {
        let mut assessment = SecurityAssessment {
            overall_score: 0.0,
            provider_scores: HashMap::new(),
            violations: Vec::new(),
            recommendations: Vec::new(),
        };

        let mut total_score = 0.0;
        let mut provider_count = 0;

        // AWS security assessment
        if let Ok(aws_client) = self.aws() {
            if let Ok(aws_assessment) = aws_client.security_assessment().await {
                assessment
                    .provider_scores
                    .insert(CloudProvider::AWS, aws_assessment.overall_score);
                assessment.violations.extend(aws_assessment.violations);
                assessment
                    .recommendations
                    .extend(aws_assessment.recommendations);
                total_score += aws_assessment.overall_score;
                provider_count += 1;
            }
        }

        // Azure security assessment
        if let Ok(azure_client) = self.azure() {
            if let Ok(azure_assessment) = azure_client.security_assessment().await {
                assessment
                    .provider_scores
                    .insert(CloudProvider::Azure, azure_assessment.overall_score);
                assessment.violations.extend(azure_assessment.violations);
                assessment
                    .recommendations
                    .extend(azure_assessment.recommendations);
                total_score += azure_assessment.overall_score;
                provider_count += 1;
            }
        }

        // GCP security assessment
        if let Ok(gcp_client) = self.gcp() {
            if let Ok(gcp_assessment) = gcp_client.security_assessment().await {
                assessment
                    .provider_scores
                    .insert(CloudProvider::GCP, gcp_assessment.overall_score);
                assessment.violations.extend(gcp_assessment.violations);
                assessment
                    .recommendations
                    .extend(gcp_assessment.recommendations);
                total_score += gcp_assessment.overall_score;
                provider_count += 1;
            }
        }

        if provider_count > 0 {
            assessment.overall_score = total_score / provider_count as f64;
        }

        Ok(assessment)
    }

    /// Generate cost optimization recommendations
    pub async fn cost_optimization(&self) -> Result<CostOptimization> {
        let mut optimization = CostOptimization {
            total_potential_savings: 0.0,
            recommendations: Vec::new(),
            rightsizing_opportunities: Vec::new(),
            reserved_instance_recommendations: Vec::new(),
        };

        // AWS cost optimization
        if let Ok(aws_client) = self.aws() {
            if let Ok(aws_optimization) = aws_client.cost_optimization().await {
                optimization.total_potential_savings += aws_optimization.total_potential_savings;
                optimization
                    .recommendations
                    .extend(aws_optimization.recommendations);
                optimization
                    .rightsizing_opportunities
                    .extend(aws_optimization.rightsizing_opportunities);
                optimization
                    .reserved_instance_recommendations
                    .extend(aws_optimization.reserved_instance_recommendations);
            }
        }

        // Azure cost optimization
        if let Ok(azure_client) = self.azure() {
            if let Ok(azure_optimization) = azure_client.cost_optimization().await {
                optimization.total_potential_savings += azure_optimization.total_potential_savings;
                optimization
                    .recommendations
                    .extend(azure_optimization.recommendations);
                optimization
                    .rightsizing_opportunities
                    .extend(azure_optimization.rightsizing_opportunities);
                optimization
                    .reserved_instance_recommendations
                    .extend(azure_optimization.reserved_instance_recommendations);
            }
        }

        // GCP cost optimization
        if let Ok(gcp_client) = self.gcp() {
            if let Ok(gcp_optimization) = gcp_client.cost_optimization().await {
                optimization.total_potential_savings += gcp_optimization.total_potential_savings;
                optimization
                    .recommendations
                    .extend(gcp_optimization.recommendations);
                optimization
                    .rightsizing_opportunities
                    .extend(gcp_optimization.rightsizing_opportunities);
                optimization
                    .reserved_instance_recommendations
                    .extend(gcp_optimization.reserved_instance_recommendations);
            }
        }

        Ok(optimization)
    }

    /// Get configuration
    pub fn get_config(&self) -> &CloudConfig {
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

/// Security assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    /// Overall security score (0-100)
    pub overall_score: f64,
    /// Scores per provider
    pub provider_scores: HashMap<CloudProvider, f64>,
    /// Security violations
    pub violations: Vec<SecurityViolation>,
    /// Security recommendations
    pub recommendations: Vec<SecurityRecommendation>,
}

/// Security violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    /// Resource ID
    pub resource_id: String,
    /// Rule ID
    pub rule_id: String,
    /// Severity
    pub severity: ViolationSeverity,
    /// Description
    pub description: String,
    /// Provider
    pub provider: CloudProvider,
}

/// Security recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    /// Recommendation ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Priority
    pub priority: RecommendationPriority,
    /// Estimated impact
    pub impact: String,
    /// Implementation steps
    pub steps: Vec<String>,
}

/// Recommendation priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Cost optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    /// Total potential savings
    pub total_potential_savings: f64,
    /// Cost recommendations
    pub recommendations: Vec<CostRecommendation>,
    /// Right-sizing opportunities
    pub rightsizing_opportunities: Vec<RightsizingRecommendation>,
    /// Reserved instance recommendations
    pub reserved_instance_recommendations: Vec<ReservedInstanceRecommendation>,
}

/// Cost recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecommendation {
    /// Resource ID
    pub resource_id: String,
    /// Recommendation type
    pub recommendation_type: String,
    /// Potential savings
    pub potential_savings: f64,
    /// Description
    pub description: String,
    /// Implementation complexity
    pub complexity: ComplexityLevel,
}

/// Right-sizing recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsizingRecommendation {
    /// Resource ID
    pub resource_id: String,
    /// Current instance type
    pub current_type: String,
    /// Recommended instance type
    pub recommended_type: String,
    /// Potential monthly savings
    pub monthly_savings: f64,
    /// CPU utilization
    pub cpu_utilization: f64,
    /// Memory utilization
    pub memory_utilization: f64,
}

/// Reserved instance recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservedInstanceRecommendation {
    /// Instance type
    pub instance_type: String,
    /// Recommended quantity
    pub quantity: u32,
    /// Term length
    pub term: ReservedInstanceTerm,
    /// Payment option
    pub payment_option: PaymentOption,
    /// Estimated annual savings
    pub annual_savings: f64,
}

/// Reserved instance term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReservedInstanceTerm {
    OneYear,
    ThreeYear,
}

/// Payment option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentOption {
    AllUpfront,
    PartialUpfront,
    NoUpfront,
}

/// Implementation complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Default implementation
impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            default_provider: CloudProvider::AWS,
            aws: None,
            azure: None,
            gcp: None,
            security: CloudSecurityConfig::default(),
            cost_management: CostManagementConfig::default(),
            governance: GovernanceConfig::default(),
        }
    }
}

impl Default for CloudSecurityConfig {
    fn default() -> Self {
        Self {
            security_scanning: true,
            compliance_monitoring: true,
            frameworks: vec![SecurityFramework::CIS, SecurityFramework::NIST],
            siem_integration: false,
            zero_trust: ZeroTrustConfig::default(),
            iam_policies: Vec::new(),
        }
    }
}

impl Default for ZeroTrustConfig {
    fn default() -> Self {
        Self {
            networking: true,
            identity: true,
            device_trust: false,
            conditional_access: Vec::new(),
        }
    }
}

impl Default for CostManagementConfig {
    fn default() -> Self {
        Self {
            cost_optimization: true,
            budget_alerts: Vec::new(),
            finops_enabled: false,
            cost_tags: HashMap::new(),
            right_sizing: true,
            reserved_instances: ReservedInstanceConfig::default(),
        }
    }
}

impl Default for ReservedInstanceConfig {
    fn default() -> Self {
        Self {
            recommendations: true,
            auto_purchase: false,
            coverage_targets: HashMap::new(),
        }
    }
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            policy_as_code: true,
            compliance_frameworks: vec![ComplianceFramework::SOC2],
            tagging_policies: Vec::new(),
            audit_logging: true,
            data_governance: DataGovernanceConfig::default(),
        }
    }
}

impl Default for DataGovernanceConfig {
    fn default() -> Self {
        Self {
            data_classification: true,
            data_lineage: false,
            retention_policies: Vec::new(),
            pii_detection: true,
        }
    }
}

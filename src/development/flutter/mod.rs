use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use tokio::process::Command as TokioCommand;
use std::path::{Path, PathBuf};

/// Flutter build target
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlutterBuildTarget {
    /// Android APK
    #[serde(rename = "apk")]
    Apk,
    /// iOS
    #[serde(rename = "ios")]
    Ios,
    /// Web
    #[serde(rename = "web")]
    Web,
    /// macOS
    #[serde(rename = "macos")]
    MacOS,
    /// Windows
    #[serde(rename = "windows")]
    Windows,
    /// Linux
    #[serde(rename = "linux")]
    Linux,
}

impl FlutterBuildTarget {
    /// Convert to string representation for CLI commands
    pub fn to_string(&self) -> String {
        match self {
            FlutterBuildTarget::Apk => "apk".to_string(),
            FlutterBuildTarget::Ios => "ios".to_string(),
            FlutterBuildTarget::Web => "web".to_string(),
            FlutterBuildTarget::MacOS => "macos".to_string(),
            FlutterBuildTarget::Windows => "windows".to_string(),
            FlutterBuildTarget::Linux => "linux".to_string(),
        }
    }
}

/// Flutter command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlutterCommandResult {
    /// Whether the command was successful
    pub success: bool,
    /// Command that was executed
    pub command: String,
    /// Command output
    pub output: String,
    /// Error output (if any)
    pub error: Option<String>,
}

/// Flutter client
pub struct FlutterClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
    /// Flutter project path
    project_path: PathBuf,
}

impl<'a> FlutterClient<'a> {
    /// Create a new Flutter client
    pub fn new(lifecycle: &'a LifecycleManager, project_path: impl AsRef<Path>) -> Result<Self> {
        // Check if Flutter is available
        Self::check_flutter()?;
        
        let project_path = project_path.as_ref().to_path_buf();
        
        // Check if the project path exists
        if !project_path.exists() {
            return Err(Error::config(format!("Project path does not exist: {}", project_path.display())));
        }
        
        // Check if it's a Flutter project (has pubspec.yaml)
        if !project_path.join("pubspec.yaml").exists() {
            return Err(Error::config(format!("Not a Flutter project, pubspec.yaml not found in: {}", project_path.display())));
        }
        
        Ok(Self {
            lifecycle,
            project_path,
        })
    }
    
    /// Check if Flutter is available
    fn check_flutter() -> Result<()> {
        match Command::new("flutter").arg("--version").stdout(Stdio::null()).status() {
            Ok(status) if status.success() => Ok(()),
            _ => Err(Error::config("Flutter not found or not properly configured".to_string())),
        }
    }
    
    /// Run a Flutter application
    pub async fn run(&self, target: Option<&str>) -> Result<FlutterCommandResult> {
        let target = target.unwrap_or("lib/main.dart");
        
        let mut cmd = TokioCommand::new("flutter");
        cmd.arg("run")
           .arg(target)
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Running Flutter application with target: {}", target);
        
        self.execute_command(cmd, format!("flutter run {}", target)).await
    }
    
    /// Generate Dart files using build_runner
    pub async fn generate(&self) -> Result<FlutterCommandResult> {
        let mut cmd = TokioCommand::new("flutter");
        cmd.arg("pub")
           .arg("run")
           .arg("build_runner")
           .arg("build")
           .arg("--delete-conflicting-outputs")
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Generating Dart files using build_runner");
        
        self.execute_command(cmd, "flutter pub run build_runner build --delete-conflicting-outputs".to_string()).await
    }
    
    /// Fix Dart style issues
    pub async fn fix(&self, folder: Option<&str>) -> Result<FlutterCommandResult> {
        let folder = folder.unwrap_or("lib");
        
        let mut cmd = TokioCommand::new("dart");
        cmd.arg("fix")
           .arg("--apply")
           .arg(folder)
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Fixing Dart style issues in folder: {}", folder);
        
        self.execute_command(cmd, format!("dart fix --apply {}", folder)).await
    }
    
    /// Run a Dart file
    pub async fn run_dart(&self, file: &str) -> Result<FlutterCommandResult> {
        let mut cmd = TokioCommand::new("dart");
        cmd.arg("run")
           .arg(file)
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Running Dart file: {}", file);
        
        self.execute_command(cmd, format!("dart run {}", file)).await
    }
    
    /// Build a Flutter application
    pub async fn build(&self, target: Option<FlutterBuildTarget>) -> Result<FlutterCommandResult> {
        let target = target.unwrap_or(FlutterBuildTarget::Apk);
        
        let mut cmd = TokioCommand::new("flutter");
        cmd.arg("build")
           .arg(target.to_string())
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Building Flutter application for target: {:?}", target);
        
        self.execute_command(cmd, format!("flutter build {}", target.to_string())).await
    }
    
    /// Run Flutter analyzer
    pub async fn analyze(&self) -> Result<FlutterCommandResult> {
        let mut cmd = TokioCommand::new("flutter");
        cmd.arg("analyze")
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Running Flutter analyzer");
        
        self.execute_command(cmd, "flutter analyze".to_string()).await
    }
    
    /// Run Flutter tests
    pub async fn test(&self, test_file: Option<&str>) -> Result<FlutterCommandResult> {
        let mut cmd = TokioCommand::new("flutter");
        cmd.arg("test");
        
        if let Some(file) = test_file {
            cmd.arg(file);
        }
        
        cmd.current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        let command_str = if let Some(file) = test_file {
            format!("flutter test {}", file)
        } else {
            "flutter test".to_string()
        };
        
        log::info!("Running Flutter tests{}", if let Some(file) = test_file { format!(": {}", file) } else { "".to_string() });
        
        self.execute_command(cmd, command_str).await
    }
    
    /// Get installed Flutter packages
    pub async fn get_packages(&self) -> Result<FlutterCommandResult> {
        let mut cmd = TokioCommand::new("flutter");
        cmd.arg("pub")
           .arg("deps")
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Getting installed Flutter packages");
        
        self.execute_command(cmd, "flutter pub deps".to_string()).await
    }
    
    /// Update Flutter packages
    pub async fn update_packages(&self) -> Result<FlutterCommandResult> {
        let mut cmd = TokioCommand::new("flutter");
        cmd.arg("pub")
           .arg("get")
           .current_dir(&self.project_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        log::info!("Updating Flutter packages");
        
        self.execute_command(cmd, "flutter pub get".to_string()).await
    }
    
    /// Execute a command and return the result
    async fn execute_command(&self, mut cmd: TokioCommand, command_str: String) -> Result<FlutterCommandResult> {
        let output = cmd.output().await
            .map_err(|e| Error::internal(format!("Failed to execute command: {}", e)))?;
            
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if output.status.success() {
            Ok(FlutterCommandResult {
                success: true,
                command: command_str,
                output: stdout,
                error: None,
            })
        } else {
            Ok(FlutterCommandResult {
                success: false,
                command: command_str,
                output: stdout,
                error: Some(stderr),
            })
        }
    }
} 
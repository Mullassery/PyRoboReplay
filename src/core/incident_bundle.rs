use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a single incident bundle (ZIP package with evidence from all layers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentBundle {
    /// Unique identifier for this bundle
    pub bundle_id: String,

    /// Path to the bundle file
    pub bundle_path: PathBuf,

    /// Auto-generated manifest with layer detection results
    pub manifest: BundleManifest,
}

/// Manifest describing what evidence is available in the bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Unique bundle identifier
    pub bundle_id: String,

    /// When the bundle was created
    pub created_at: DateTime<Utc>,

    /// ID of the robot(s) involved
    pub robot_ids: Vec<String>,

    /// Type of mission/failure suspected
    pub mission_type: Option<String>,
    pub failure_type_suspected: Option<String>,

    /// Time range covered by the bundle
    pub time_range: Option<TimeRange>,

    /// Which layers are available
    pub layers_available: LayerAvailability,

    /// Detected issues from quick scan
    pub detected_issues: Vec<String>,

    /// File inventory by layer
    pub file_inventory: LayerFileInventory,

    /// Checksums for integrity verification
    pub checksums: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    pub fn duration_seconds(&self) -> i64 {
        (self.end - self.start).num_seconds()
    }
}

/// Which layers of evidence are present in this bundle
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayerAvailability {
    /// Layer 1: ROS bags and logs (minimum required)
    pub layer1_ros_bags: bool,

    /// Layer 2: Linux system logs
    pub layer2_linux_logs: bool,

    /// Layer 3: Resource metrics
    pub layer3_metrics: bool,

    /// Layer 4: Configuration files
    pub layer4_configs: bool,
}

impl LayerAvailability {
    pub fn available_layer_count(&self) -> u32 {
        let mut count = 0;
        if self.layer1_ros_bags { count += 1; }
        if self.layer2_linux_logs { count += 1; }
        if self.layer3_metrics { count += 1; }
        if self.layer4_configs { count += 1; }
        count
    }

    pub fn analysis_level(&self) -> u32 {
        self.available_layer_count()
    }
}

/// Inventory of files organized by layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayerFileInventory {
    /// Layer 1 files: ROS bags, ROS logs, TF frames
    pub layer1: Layer1Files,

    /// Layer 2 files: journalctl, dmesg, syslog, kernel logs
    pub layer2: Layer2Files,

    /// Layer 3 files: metrics CSV, DDS telemetry, network data
    pub layer3: Layer3Files,

    /// Layer 4 files: YAML configs, launch files, hardware config
    pub layer4: Layer4Files,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Layer1Files {
    pub ros_bags: Vec<PathBuf>,
    pub node_logs: Vec<PathBuf>,
    pub tf_frames_log: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Layer2Files {
    pub journalctl_log: Option<PathBuf>,
    pub dmesg_log: Option<PathBuf>,
    pub syslog: Option<PathBuf>,
    pub kernel_logs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Layer3Files {
    pub cpu_metrics: Option<PathBuf>,
    pub memory_metrics: Option<PathBuf>,
    pub disk_metrics: Option<PathBuf>,
    pub thermal_metrics: Option<PathBuf>,
    pub network_metrics: Option<PathBuf>,
    pub dds_telemetry: Option<PathBuf>,
    pub other_metrics: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Layer4Files {
    pub nav2_yaml: Option<PathBuf>,
    pub slam_yaml: Option<PathBuf>,
    pub launch_files: Vec<PathBuf>,
    pub hardware_config: Option<PathBuf>,
    pub other_configs: Vec<PathBuf>,
}

/// Error type for bundle operations
#[derive(Debug)]
pub enum BundleError {
    ZipError(String),
    IoError(String),
    ManifestError(String),
    InvalidBundle(String),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::ZipError(e) => write!(f, "ZIP error: {}", e),
            BundleError::IoError(e) => write!(f, "IO error: {}", e),
            BundleError::ManifestError(e) => write!(f, "Manifest error: {}", e),
            BundleError::InvalidBundle(e) => write!(f, "Invalid bundle: {}", e),
        }
    }
}

impl std::error::Error for BundleError {}

impl IncidentBundle {
    /// Create a new bundle from a ZIP file path
    pub fn from_zip(bundle_path: &Path) -> Result<Self, BundleError> {
        if !bundle_path.exists() {
            return Err(BundleError::InvalidBundle(format!(
                "Bundle file not found: {:?}",
                bundle_path
            )));
        }

        let bundle_id = bundle_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("incident")
            .to_string();

        Ok(IncidentBundle {
            bundle_id: bundle_id.clone(),
            bundle_path: bundle_path.to_path_buf(),
            manifest: BundleManifest {
                bundle_id,
                created_at: Utc::now(),
                robot_ids: Vec::new(),
                mission_type: None,
                failure_type_suspected: None,
                time_range: None,
                layers_available: LayerAvailability::default(),
                detected_issues: Vec::new(),
                file_inventory: LayerFileInventory::default(),
                checksums: HashMap::new(),
            },
        })
    }

    /// Get path to a layer directory within the bundle
    pub fn get_layer_path(&self, layer: u32) -> PathBuf {
        match layer {
            1 => self.bundle_path.parent().unwrap_or_else(|| Path::new("")).join("layer1"),
            2 => self.bundle_path.parent().unwrap_or_else(|| Path::new("")).join("layer2"),
            3 => self.bundle_path.parent().unwrap_or_else(|| Path::new("")).join("layer3"),
            4 => self.bundle_path.parent().unwrap_or_else(|| Path::new("")).join("layer4"),
            _ => PathBuf::new(),
        }
    }

    /// Check which layers are available in this bundle
    pub fn analyze_layers(&mut self) -> Result<LayerAvailability, BundleError> {
        // This will be implemented by the evidence discovery module
        // For now, return the current state
        Ok(self.manifest.layers_available.clone())
    }

    /// Get the analysis level (how many layers available)
    pub fn analysis_level(&self) -> u32 {
        self.manifest.layers_available.analysis_level()
    }

    /// Generate a human-readable summary of the bundle
    pub fn summary(&self) -> String {
        let level = self.analysis_level();
        let robot_list = self.manifest.robot_ids.join(", ");

        let mut summary = format!(
            "Incident Bundle: {}\n\
             Created: {}\n\
             Analysis Level: {}/4\n\
             Robots: {}\n\
             Available Evidence:\n",
            self.manifest.bundle_id,
            self.manifest.created_at.format("%Y-%m-%d %H:%M:%S"),
            level,
            robot_list
        );

        let avail = &self.manifest.layers_available;
        if avail.layer1_ros_bags {
            summary.push_str("  ✓ Layer 1: ROS Bags & Logs\n");
        }
        if avail.layer2_linux_logs {
            summary.push_str("  ✓ Layer 2: Linux/Kernel Logs\n");
        }
        if avail.layer3_metrics {
            summary.push_str("  ✓ Layer 3: Resource Metrics\n");
        }
        if avail.layer4_configs {
            summary.push_str("  ✓ Layer 4: Configuration Files\n");
        }

        if !self.manifest.detected_issues.is_empty() {
            summary.push_str("\nDetected Issues:\n");
            for issue in &self.manifest.detected_issues {
                summary.push_str(&format!("  • {}\n", issue));
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_creation() {
        let bundle = IncidentBundle::from_zip(Path::new("test.zip"));
        assert!(bundle.is_ok());
    }

    #[test]
    fn test_time_range_duration() {
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(900);
        let time_range = TimeRange { start, end };
        assert_eq!(time_range.duration_seconds(), 900);
    }

    #[test]
    fn test_layer_availability_count() {
        let mut avail = LayerAvailability::default();
        assert_eq!(avail.available_layer_count(), 0);

        avail.layer1_ros_bags = true;
        assert_eq!(avail.available_layer_count(), 1);

        avail.layer2_linux_logs = true;
        assert_eq!(avail.available_layer_count(), 2);
    }
}

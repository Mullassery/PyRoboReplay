use crate::core::incident_bundle::{
    IncidentBundle, LayerFileInventory,
    Layer1Files, Layer2Files, Layer3Files, Layer4Files, BundleError, TimeRange,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Auto-discovers evidence in an incident bundle
pub struct EvidenceDiscovery;

impl EvidenceDiscovery {
    /// Scan a directory and auto-discover available evidence layers
    pub fn discover(bundle_path: &Path) -> Result<IncidentBundle, BundleError> {
        let mut bundle = IncidentBundle::from_zip(bundle_path)?;

        // Detect Layer 1 (ROS bags and logs)
        let layer1 = Self::discover_layer1(bundle_path)?;
        bundle.manifest.layers_available.layer1_ros_bags = !layer1.ros_bags.is_empty()
            || !layer1.node_logs.is_empty()
            || layer1.tf_frames_log.is_some();

        // Detect Layer 2 (Linux logs)
        let layer2 = Self::discover_layer2(bundle_path)?;
        bundle.manifest.layers_available.layer2_linux_logs = !layer2.journalctl_log.is_none()
            || !layer2.dmesg_log.is_none()
            || !layer2.syslog.is_none()
            || !layer2.kernel_logs.is_empty();

        // Detect Layer 3 (Metrics)
        let layer3 = Self::discover_layer3(bundle_path)?;
        bundle.manifest.layers_available.layer3_metrics = layer3.cpu_metrics.is_some()
            || layer3.memory_metrics.is_some()
            || layer3.thermal_metrics.is_some()
            || layer3.network_metrics.is_some()
            || layer3.dds_telemetry.is_some()
            || !layer3.other_metrics.is_empty();

        // Detect Layer 4 (Configs)
        let layer4 = Self::discover_layer4(bundle_path)?;
        bundle.manifest.layers_available.layer4_configs = layer4.nav2_yaml.is_some()
            || layer4.slam_yaml.is_some()
            || !layer4.launch_files.is_empty()
            || layer4.hardware_config.is_some()
            || !layer4.other_configs.is_empty();

        // Store file inventory
        bundle.manifest.file_inventory = LayerFileInventory {
            layer1,
            layer2,
            layer3,
            layer4,
        };

        // Extract robot IDs from file names and logs
        bundle.manifest.robot_ids = Self::extract_robot_ids(&bundle)?;

        // Detect time range
        bundle.manifest.time_range = Self::extract_time_range(&bundle)?;

        // Quick scan for common issues
        bundle.manifest.detected_issues = Self::detect_quick_issues(&bundle)?;

        Ok(bundle)
    }

    /// Discover Layer 1: ROS bags and logs
    fn discover_layer1(bundle_path: &Path) -> Result<Layer1Files, BundleError> {
        let mut layer1 = Layer1Files::default();
        let layer_dir = bundle_path.join("layer1");

        if layer_dir.exists() {
            for entry in fs::read_dir(&layer_dir)
                .map_err(|e| BundleError::IoError(e.to_string()))?
            {
                let entry = entry.map_err(|e| BundleError::IoError(e.to_string()))?;
                let path = entry.path();

                if let Some(ext) = path.extension() {
                    match ext.to_str().unwrap_or("") {
                        "bag" | "db3" => layer1.ros_bags.push(path),
                        "log" => {
                            if path.file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.contains("tf_frames"))
                                .unwrap_or(false)
                            {
                                layer1.tf_frames_log = Some(path);
                            } else if path.file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.contains("node") || s.contains("robot"))
                                .unwrap_or(false)
                            {
                                layer1.node_logs.push(path);
                            } else {
                                layer1.node_logs.push(path);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(layer1)
    }

    /// Discover Layer 2: Linux/kernel logs
    fn discover_layer2(bundle_path: &Path) -> Result<Layer2Files, BundleError> {
        let mut layer2 = Layer2Files::default();
        let layer_dir = bundle_path.join("layer2");

        if layer_dir.exists() {
            for entry in fs::read_dir(&layer_dir)
                .map_err(|e| BundleError::IoError(e.to_string()))?
            {
                let entry = entry.map_err(|e| BundleError::IoError(e.to_string()))?;
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    match name {
                        "journalctl.log" => layer2.journalctl_log = Some(path),
                        "dmesg.log" => layer2.dmesg_log = Some(path),
                        "syslog.log" => layer2.syslog = Some(path),
                        _ if name.contains("kernel") => layer2.kernel_logs.push(path),
                        _ if name.ends_with(".log") => layer2.kernel_logs.push(path),
                        _ => {}
                    }
                }
            }
        }

        Ok(layer2)
    }

    /// Discover Layer 3: Resource metrics
    fn discover_layer3(bundle_path: &Path) -> Result<Layer3Files, BundleError> {
        let mut layer3 = Layer3Files::default();
        let layer_dir = bundle_path.join("layer3");

        if layer_dir.exists() {
            for entry in fs::read_dir(&layer_dir)
                .map_err(|e| BundleError::IoError(e.to_string()))?
            {
                let entry = entry.map_err(|e| BundleError::IoError(e.to_string()))?;
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    match name {
                        "cpu_metrics.csv" | "cpu.csv" | "cpu_percent.csv" => layer3.cpu_metrics = Some(path),
                        "memory_metrics.csv" | "memory.csv" | "memory_mb.csv" => layer3.memory_metrics = Some(path),
                        "disk_metrics.csv" | "disk.csv" | "disk_percent.csv" => layer3.disk_metrics = Some(path),
                        "thermal_metrics.csv" | "thermal.csv" | "temperature.csv" | "temp.csv" => layer3.thermal_metrics = Some(path),
                        "network_metrics.csv" | "network.csv" | "network_io.csv" => layer3.network_metrics = Some(path),
                        "dds_metrics.json" | "dds_telemetry.json" => layer3.dds_telemetry = Some(path),
                        _ if name.ends_with(".csv") || name.ends_with(".json") => {
                            layer3.other_metrics.push(path);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(layer3)
    }

    /// Discover Layer 4: Configuration files
    fn discover_layer4(bundle_path: &Path) -> Result<Layer4Files, BundleError> {
        let mut layer4 = Layer4Files::default();
        let layer_dir = bundle_path.join("layer4");

        if layer_dir.exists() {
            for entry in fs::read_dir(&layer_dir)
                .map_err(|e| BundleError::IoError(e.to_string()))?
            {
                let entry = entry.map_err(|e| BundleError::IoError(e.to_string()))?;
                let path = entry.path();

                if path.is_dir() && path.file_name().and_then(|n| n.to_str()).map(|s| s == "launch_files").unwrap_or(false) {
                    if let Ok(launches) = fs::read_dir(&path) {
                        for launch_entry in launches.flatten() {
                            let launch_path = launch_entry.path();
                            if launch_path.extension().and_then(|e| e.to_str()).map(|e| e == "py" || e == "xml").unwrap_or(false) {
                                layer4.launch_files.push(launch_path);
                            }
                        }
                    }
                    continue;
                }

                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    match name {
                        "nav2_params.yaml" | "nav2.yaml" => layer4.nav2_yaml = Some(path),
                        "slam_params.yaml" | "slam.yaml" => layer4.slam_yaml = Some(path),
                        "hardware_config.yaml" | "hardware.yaml" => layer4.hardware_config = Some(path),
                        _ if name.ends_with(".yaml") || name.ends_with(".yml") => {
                            layer4.other_configs.push(path);
                        }
                        _ if name.ends_with(".py") || name.ends_with(".xml") => {
                            layer4.launch_files.push(path);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(layer4)
    }

    /// Extract robot IDs from file names and metadata
    fn extract_robot_ids(bundle: &IncidentBundle) -> Result<Vec<String>, BundleError> {
        let mut robot_ids = Vec::new();

        // Try to extract from ROS bag names
        for bag_path in &bundle.manifest.file_inventory.layer1.ros_bags {
            if let Some(file_name) = bag_path.file_stem().and_then(|n| n.to_str()) {
                // Common patterns: robot1, robot_1, r1, etc.
                if let Some(robot_part) = file_name.split('_').find(|s| s.starts_with("robot")) {
                    robot_ids.push(robot_part.to_string());
                }
            }
        }

        // Fallback: generic robot ID if none found
        if robot_ids.is_empty() {
            robot_ids.push("robot1".to_string());
        }

        Ok(robot_ids)
    }

    /// Extract time range from logs
    fn extract_time_range(_bundle: &IncidentBundle) -> Result<Option<TimeRange>, BundleError> {
        // This would parse actual log files to find time range
        // For now, return None to indicate it needs to be determined during analysis
        Ok(None)
    }

    /// Quick scan for common issues
    fn detect_quick_issues(bundle: &IncidentBundle) -> Result<Vec<String>, BundleError> {
        let mut issues = Vec::new();

        // Check Layer 2 for known failure patterns
        if bundle.manifest.layers_available.layer2_linux_logs {
            if bundle.manifest.file_inventory.layer2.dmesg_log.is_some() {
                // Would parse dmesg for OOM, kernel panic, etc.
                issues.push("kernel_log_present".to_string());
            }
        }

        // Check Layer 3 for resource anomalies
        if bundle.manifest.layers_available.layer3_metrics {
            if bundle.manifest.file_inventory.layer3.cpu_metrics.is_some() {
                // Would scan CPU metrics for spikes
                issues.push("cpu_metrics_available".to_string());
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer1_discovery() {
        let layer1 = Layer1Files::default();
        assert_eq!(layer1.ros_bags.len(), 0);
        assert_eq!(layer1.node_logs.len(), 0);
    }

    #[test]
    fn test_robot_id_extraction() {
        // Resolve the fixture relative to the crate manifest dir (not the
        // process CWD) so this test doesn't depend on where `cargo test` is
        // invoked from.
        let fixture = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.zip"));
        let mut bundle = IncidentBundle::from_zip(fixture).unwrap();
        bundle.manifest.file_inventory.layer1.ros_bags.push(PathBuf::from("robot1_mission.bag"));

        let robot_ids = EvidenceDiscovery::extract_robot_ids(&bundle).unwrap();
        assert!(!robot_ids.is_empty());
    }
}

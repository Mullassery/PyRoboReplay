//! Thermal Imaging Model & Processing
//!
//! Represents thermal/infrared sensor data and enables fusion with RGB imagery.
//! Thermal detects heat signatures invisible to RGB in low-light, fog, smoke, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Thermal camera configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalCameraConfig {
    /// Camera resolution (width, height)
    pub resolution: (u32, u32),
    /// Thermal range (min, max) in Kelvin
    pub temp_range_k: (f32, f32),
    /// Pixel thermal sensitivity (K per DN unit)
    pub sensitivity_k_per_dn: f32,
    /// NETD (Noise Equivalent Temperature Difference) in K
    pub netd_k: f32,
    /// Field of view (degrees)
    pub fov_degrees: f32,
}

impl Default for ThermalCameraConfig {
    fn default() -> Self {
        ThermalCameraConfig {
            resolution: (640, 480),
            temp_range_k: (273.15, 473.15), // 0°C to 200°C
            sensitivity_k_per_dn: 0.04,
            netd_k: 0.1,
            fov_degrees: 45.0,
        }
    }
}

/// Thermal frame from infrared sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalFrame {
    /// Frame timestamp
    pub timestamp_sec: f32,
    /// Frame index
    pub frame_index: usize,
    /// Raw thermal data (temperature in Kelvin)
    pub thermal_data: Vec<f32>, // width * height elements
    /// Width of frame
    pub width: u32,
    /// Height of frame
    pub height: u32,
    /// Configuration used
    pub config: ThermalCameraConfig,
}

impl ThermalFrame {
    /// Get pixel temperature at (x, y)
    pub fn get_pixel_temp(&self, x: u32, y: u32) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.thermal_data.get(idx).copied()
    }

    /// Get average temperature in region
    pub fn get_region_avg(&self, x: u32, y: u32, width: u32, height: u32) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;

        for ry in y..y.saturating_add(height).min(self.height) {
            for rx in x..x.saturating_add(width).min(self.width) {
                if let Some(temp) = self.get_pixel_temp(rx, ry) {
                    sum += temp;
                    count += 1;
                }
            }
        }

        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }

    /// Get max temperature in region (hotspot)
    pub fn get_region_max(&self, x: u32, y: u32, width: u32, height: u32) -> f32 {
        let mut max_temp = f32::NEG_INFINITY;

        for ry in y..y.saturating_add(height).min(self.height) {
            for rx in x..x.saturating_add(width).min(self.width) {
                if let Some(temp) = self.get_pixel_temp(rx, ry) {
                    max_temp = max_temp.max(temp);
                }
            }
        }

        max_temp
    }

    /// Identify thermal hotspots (anomalies)
    pub fn detect_hotspots(&self, threshold_above_ambient_k: f32) -> Vec<ThermalHotspot> {
        let global_avg = self.thermal_data.iter().sum::<f32>() / self.thermal_data.len() as f32;
        let hotspot_threshold = global_avg + threshold_above_ambient_k;

        let mut hotspots = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for (idx, &temp) in self.thermal_data.iter().enumerate() {
            if temp > hotspot_threshold && !visited.contains(&idx) {
                // Flood fill to find connected hotspot
                let (x, y) = ((idx as u32) % self.width, (idx as u32) / self.width);
                let hotspot = self.extract_hotspot(x, y, hotspot_threshold, &mut visited);
                if hotspot.pixel_count > 10 {
                    // Only report if >10 pixels
                    hotspots.push(hotspot);
                }
            }
        }

        hotspots
    }

    fn extract_hotspot(
        &self,
        start_x: u32,
        start_y: u32,
        threshold: f32,
        visited: &mut std::collections::HashSet<usize>,
    ) -> ThermalHotspot {
        let mut pixels = Vec::new();
        let mut queue = vec![(start_x, start_y)];
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_temp = 0.0;
        let mut max_temp = f32::NEG_INFINITY;

        while let Some((x, y)) = queue.pop() {
            if x >= self.width || y >= self.height {
                continue;
            }

            let idx = (y * self.width + x) as usize;
            if visited.contains(&idx) {
                continue;
            }

            if let Some(temp) = self.get_pixel_temp(x, y) {
                if temp > threshold {
                    visited.insert(idx);
                    pixels.push((x, y));
                    sum_x += x as f32;
                    sum_y += y as f32;
                    sum_temp += temp;
                    max_temp = max_temp.max(temp);

                    // Add neighbors
                    if x > 0 {
                        queue.push((x - 1, y));
                    }
                    if x + 1 < self.width {
                        queue.push((x + 1, y));
                    }
                    if y > 0 {
                        queue.push((x, y - 1));
                    }
                    if y + 1 < self.height {
                        queue.push((x, y + 1));
                    }
                }
            }
        }

        let pixel_count = pixels.len() as f32;
        let center_x = sum_x / pixel_count;
        let center_y = sum_y / pixel_count;
        let avg_temp = sum_temp / pixel_count;

        ThermalHotspot {
            center_x,
            center_y,
            avg_temp_k: avg_temp,
            max_temp_k: max_temp,
            pixel_count: pixels.len(),
            pixels,
        }
    }

    /// Estimate human presence likelihood based on thermal signature
    pub fn estimate_human_likelihood(&self, region_x: u32, region_y: u32, region_w: u32, region_h: u32) -> f32 {
        let avg_temp = self.get_region_avg(region_x, region_y, region_w, region_h);
        let max_temp = self.get_region_max(region_x, region_y, region_w, region_h);

        // Human body temperature ~310K (37°C), clothes ~300-310K
        let core_temp = 310.0;
        let surface_temp = 305.0;

        // Likelihood increases near body temperature
        let temp_diff_max = (max_temp - surface_temp).abs();
        let temp_diff_avg = (avg_temp - surface_temp).abs();

        // Strong signature if within ±10K of expected
        if temp_diff_max < 10.0 && temp_diff_avg < 5.0 {
            0.95 // Strong human signature
        } else if temp_diff_max < 15.0 {
            0.7 // Moderate signature
        } else if temp_diff_max < 25.0 {
            0.4 // Weak signature (could be human)
        } else {
            0.0 // No thermal signature
        }
    }
}

/// Thermal hotspot (anomalous heat signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalHotspot {
    /// Center X coordinate
    pub center_x: f32,
    /// Center Y coordinate
    pub center_y: f32,
    /// Average temperature (Kelvin)
    pub avg_temp_k: f32,
    /// Maximum temperature (Kelvin)
    pub max_temp_k: f32,
    /// Number of pixels in hotspot
    pub pixel_count: usize,
    /// Pixels in hotspot
    pub pixels: Vec<(u32, u32)>,
}

impl ThermalHotspot {
    /// Estimate likely source of heat signature
    pub fn estimate_source(&self) -> ThermalSource {
        let temp_c = self.avg_temp_k - 273.15;

        if temp_c >= 30.0 && temp_c <= 40.0 {
            // Body temperature range
            if self.pixel_count > 100 {
                ThermalSource::Human
            } else if self.pixel_count > 30 {
                ThermalSource::HumanPartial // Exposed skin
            } else {
                ThermalSource::Unknown
            }
        } else if temp_c >= 20.0 && temp_c <= 35.0 {
            ThermalSource::Animal
        } else if temp_c > 60.0 {
            ThermalSource::Engine
        } else if temp_c > 40.0 {
            ThermalSource::Machinery
        } else {
            ThermalSource::Unknown
        }
    }

    /// Confidence in source estimation
    pub fn source_confidence(&self) -> f32 {
        match self.estimate_source() {
            ThermalSource::Human => 0.9,
            ThermalSource::HumanPartial => 0.7,
            ThermalSource::Animal => 0.6,
            ThermalSource::Engine => 0.85,
            ThermalSource::Machinery => 0.75,
            _ => 0.3,
        }
    }
}

/// Identified thermal source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalSource {
    Human,
    HumanPartial,
    Animal,
    Engine,
    Machinery,
    Battery,
    ElectricalHotspot,
    RecentlyOccupied,
    Unknown,
}

impl std::fmt::Display for ThermalSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ThermalSource::Human => write!(f, "Human"),
            ThermalSource::HumanPartial => write!(f, "Human (Partial)"),
            ThermalSource::Animal => write!(f, "Animal"),
            ThermalSource::Engine => write!(f, "Engine"),
            ThermalSource::Machinery => write!(f, "Machinery"),
            ThermalSource::Battery => write!(f, "Battery"),
            ThermalSource::ElectricalHotspot => write!(f, "Electrical Hotspot"),
            ThermalSource::RecentlyOccupied => write!(f, "Recently Occupied"),
            ThermalSource::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_thermal_frame() -> ThermalFrame {
        let config = ThermalCameraConfig::default();
        let mut thermal_data = vec![290.0; (config.resolution.0 * config.resolution.1) as usize];

        // Add human-temperature region
        for y in 100..150 {
            for x in 100..150 {
                let idx = (y * config.resolution.0 + x) as usize;
                thermal_data[idx] = 305.0; // Human skin temperature
            }
        }

        ThermalFrame {
            timestamp_sec: 100.0,
            frame_index: 0,
            thermal_data,
            width: config.resolution.0,
            height: config.resolution.1,
            config,
        }
    }

    #[test]
    fn test_thermal_frame_creation() {
        let frame = create_test_thermal_frame();
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
    }

    #[test]
    fn test_get_pixel_temp() {
        let frame = create_test_thermal_frame();
        let temp = frame.get_pixel_temp(100, 100);
        assert!(temp.is_some());
        assert!((temp.unwrap() - 305.0).abs() < 0.1);
    }

    #[test]
    fn test_region_average() {
        let frame = create_test_thermal_frame();
        let avg = frame.get_region_avg(100, 100, 50, 50);
        assert!(avg > 300.0 && avg < 310.0);
    }

    #[test]
    fn test_hotspot_detection() {
        let frame = create_test_thermal_frame();
        let hotspots = frame.detect_hotspots(10.0);
        assert!(!hotspots.is_empty());
    }

    #[test]
    fn test_human_likelihood() {
        let frame = create_test_thermal_frame();
        let likelihood = frame.estimate_human_likelihood(100, 100, 50, 50);
        assert!(likelihood > 0.8);
    }

    #[test]
    fn test_hotspot_source_estimation() {
        let hotspot = ThermalHotspot {
            center_x: 125.0,
            center_y: 125.0,
            avg_temp_k: 305.0,
            max_temp_k: 310.0,
            pixel_count: 150,
            pixels: vec![],
        };

        assert_eq!(hotspot.estimate_source(), ThermalSource::Human);
    }
}

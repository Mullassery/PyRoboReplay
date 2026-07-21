/// IMU data visualization for terminal replay
/// Renders accelerometer, gyro, and magnetometer data as ASCII graphs

/// Configuration for IMU visualization
#[derive(Debug, Clone)]
pub struct IMUVizConfig {
    pub width: usize,
    pub height: usize,
    pub accel_range: f64,      // m/s^2, typically 0-20
    pub gyro_range: f64,       // rad/s, typically 0-10
    pub mag_range: f64,        // µT, typically 0-100
    pub show_stats: bool,
    pub detect_peaks: bool,
}

impl Default for IMUVizConfig {
    fn default() -> Self {
        Self {
            width: 60,
            height: 12,
            accel_range: 20.0,
            gyro_range: 10.0,
            mag_range: 100.0,
            show_stats: true,
            detect_peaks: true,
        }
    }
}

/// Peak detection result
#[derive(Debug, Clone)]
pub struct Peak {
    pub index: usize,
    pub value: f64,
    pub axis: char, // 'x', 'y', 'z'
}

/// IMU visualization with graphs and statistics
#[derive(Debug, Clone)]
pub struct IMUVisualization {
    accel_data: [Vec<f64>; 3],  // x, y, z
    gyro_data: [Vec<f64>; 3],
    mag_data: [Vec<f64>; 3],
    timestamps: Vec<String>,
    peaks: Vec<Peak>,
}

impl IMUVisualization {
    /// Create new IMU visualization
    pub fn new() -> Self {
        Self {
            accel_data: [Vec::new(), Vec::new(), Vec::new()],
            gyro_data: [Vec::new(), Vec::new(), Vec::new()],
            mag_data: [Vec::new(), Vec::new(), Vec::new()],
            timestamps: Vec::new(),
            peaks: Vec::new(),
        }
    }

    /// Add IMU reading to visualization
    pub fn add_reading(
        &mut self,
        timestamp: &str,
        accel: [f64; 3],
        gyro: [f64; 3],
        mag: Option<[f64; 3]>,
    ) {
        self.accel_data[0].push(accel[0]);
        self.accel_data[1].push(accel[1]);
        self.accel_data[2].push(accel[2]);

        self.gyro_data[0].push(gyro[0]);
        self.gyro_data[1].push(gyro[1]);
        self.gyro_data[2].push(gyro[2]);

        if let Some(m) = mag {
            self.mag_data[0].push(m[0]);
            self.mag_data[1].push(m[1]);
            self.mag_data[2].push(m[2]);
        }

        self.timestamps.push(timestamp.to_string());
    }

    /// Detect peaks (impacts/events)
    pub fn detect_peaks(&mut self, config: &IMUVizConfig) {
        if !config.detect_peaks {
            return;
        }

        self.peaks.clear();

        // Detect accel peaks (impacts)
        for (axis_char, axis_idx) in [('x', 0), ('y', 1), ('z', 2)] {
            if let Some(peaks) = Self::find_peaks(&self.accel_data[axis_idx], 2.0) {
                for (idx, val) in peaks {
                    self.peaks.push(Peak {
                        index: idx,
                        value: val,
                        axis: axis_char,
                    });
                }
            }
        }

        // Detect gyro peaks (rotation events)
        for (axis_char, axis_idx) in [('R', 0), ('P', 1), ('Y', 2)] {
            if let Some(peaks) = Self::find_peaks(&self.gyro_data[axis_idx], 1.0) {
                for (idx, val) in peaks {
                    self.peaks.push(Peak {
                        index: idx,
                        value: val,
                        axis: axis_char,
                    });
                }
            }
        }
    }

    /// Find peaks in data (simple peak detection)
    fn find_peaks(data: &[f64], threshold: f64) -> Option<Vec<(usize, f64)>> {
        if data.len() < 3 {
            return None;
        }

        let mut peaks = Vec::new();

        for i in 1..data.len() - 1 {
            let curr = data[i].abs();
            let prev = data[i - 1].abs();
            let next = data[i + 1].abs();

            // Peak if current is greater than neighbors and above threshold
            if curr > prev && curr > next && curr > threshold {
                peaks.push((i, data[i]));
            }
        }

        if peaks.is_empty() {
            None
        } else {
            Some(peaks)
        }
    }

    /// Render accelerometer graph
    pub fn render_accel(&self, config: &IMUVizConfig) -> String {
        Self::render_graph(
            "Accelerometer (m/s²)",
            &self.accel_data,
            config.width,
            config.height,
            config.accel_range,
            &['X', 'Y', 'Z'],
        )
    }

    /// Render gyroscope graph
    pub fn render_gyro(&self, config: &IMUVizConfig) -> String {
        Self::render_graph(
            "Gyroscope (rad/s)",
            &self.gyro_data,
            config.width,
            config.height,
            config.gyro_range,
            &['X', 'Y', 'Z'],
        )
    }

    /// Render magnetometer graph
    pub fn render_mag(&self, config: &IMUVizConfig) -> String {
        Self::render_graph(
            "Magnetometer (µT)",
            &self.mag_data,
            config.width,
            config.height,
            config.mag_range,
            &['X', 'Y', 'Z'],
        )
    }

    /// Generic graph rendering
    fn render_graph(
        title: &str,
        data: &[Vec<f64>; 3],
        width: usize,
        _height: usize,
        _max_range: f64,
        labels: &[char; 3],
    ) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&format!("┌{:─<width$}┐\n", format!(" {} ", title)));

        // Determine which axis has data
        let has_data = [
            !data[0].is_empty(),
            !data[1].is_empty(),
            !data[2].is_empty(),
        ];

        if !has_data.iter().any(|&x| x) {
            output.push_str("│ No data │\n");
            output.push_str(&format!("└{:─<width$}┘", ""));
            return output;
        }

        // Find data range to display
        let (min_val, max_val) = Self::find_data_range(data, &has_data);
        let actual_range = (max_val - min_val).abs().max(0.1);

        // Render each data series
        for (axis_idx, axis_label) in labels.iter().enumerate() {
            if !has_data[axis_idx] {
                continue;
            }

            output.push_str(&format!("│{} ", axis_label));

            let series = &data[axis_idx];
            let colors = ['▄', '█', '▓', '▒', '░', '·', '-', '_'];

            // Sample data to fit width
            let step = (series.len() as f64 / width as f64).max(1.0) as usize;
            let mut col = 0;

            for chunk in series.chunks(step) {
                if col >= width - 2 {
                    break;
                }

                // Get max value in chunk
                let max_in_chunk = chunk
                    .iter()
                    .map(|x| x.abs())
                    .fold(0.0f64, |a, b| a.max(b));

                // Normalize to range [0, 1]
                let normalized = if actual_range > 0.01 {
                    ((max_in_chunk - min_val.abs()) / actual_range).max(0.0).min(1.0)
                } else {
                    0.0
                };

                // Select character based on intensity
                let ch_idx = (normalized * 7.9) as usize;
                let ch = colors[ch_idx.min(7)];
                output.push(ch);
                col += 1;
            }

            // Pad to width
            for _ in col..width - 2 {
                output.push(' ');
            }

            output.push_str("│\n");
        }

        output.push_str(&format!("└{:─<width$}┘", ""));

        output
    }

    /// Find data range for scaling
    fn find_data_range(data: &[Vec<f64>; 3], has_data: &[bool; 3]) -> (f64, f64) {
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for (idx, enabled) in has_data.iter().enumerate() {
            if *enabled {
                for val in &data[idx] {
                    min = min.min(*val);
                    max = max.max(*val);
                }
            }
        }

        if min == f64::MAX {
            (0.0, 1.0)
        } else {
            (min, max)
        }
    }

    /// Calculate statistics
    pub fn stats(&self, _config: &IMUVizConfig) -> String {
        let mut output = String::new();

        output.push_str("IMU Statistics\n");
        output.push_str("├─ Accelerometer:\n");

        for (axis, label) in [(0, "X"), (1, "Y"), (2, "Z")] {
            if !self.accel_data[axis].is_empty() {
                let mean = Self::mean(&self.accel_data[axis]);
                let peak = Self::peak(&self.accel_data[axis]);
                let drift = Self::drift(&self.accel_data[axis]);
                output.push_str(&format!(
                    "│  {} → Mean: {:.2}, Peak: {:.2}, Drift: {:.2}\n",
                    label, mean, peak, drift
                ));
            }
        }

        output.push_str("├─ Gyroscope:\n");
        for (axis, label) in [(0, "Roll"), (1, "Pitch"), (2, "Yaw")] {
            if !self.gyro_data[axis].is_empty() {
                let mean = Self::mean(&self.gyro_data[axis]);
                let peak = Self::peak(&self.gyro_data[axis]);
                let drift = Self::drift(&self.gyro_data[axis]);
                output.push_str(&format!(
                    "│  {} → Mean: {:.3}, Peak: {:.3}, Drift: {:.3}\n",
                    label, mean, peak, drift
                ));
            }
        }

        if !self.mag_data[0].is_empty() {
            output.push_str("├─ Magnetometer:\n");
            for (axis, label) in [(0, "X"), (1, "Y"), (2, "Z")] {
                if !self.mag_data[axis].is_empty() {
                    let mean = Self::mean(&self.mag_data[axis]);
                    let peak = Self::peak(&self.mag_data[axis]);
                    output.push_str(&format!(
                        "│  {} → Mean: {:.1}, Peak: {:.1}\n",
                        label, mean, peak
                    ));
                }
            }
        }

        if !self.peaks.is_empty() {
            output.push_str(&format!("└─ Detected Peaks: {}\n", self.peaks.len()));
        } else {
            output.push_str("└─ No significant peaks detected\n");
        }

        output
    }

    /// Calculate mean of data
    fn mean(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        data.iter().sum::<f64>() / data.len() as f64
    }

    /// Calculate peak (max absolute value)
    fn peak(data: &[f64]) -> f64 {
        data.iter()
            .map(|x| x.abs())
            .fold(0.0f64, |a, b| a.max(b))
    }

    /// Calculate drift (variance from first value)
    fn drift(data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first = data[0];
        let last = data[data.len() - 1];
        (last - first).abs()
    }

    /// Render complete IMU dashboard
    pub fn render_dashboard(&self, config: &IMUVizConfig) -> String {
        let mut output = String::new();

        output.push_str(&self.render_accel(config));
        output.push('\n');
        output.push_str(&self.render_gyro(config));
        output.push('\n');

        if !self.mag_data[0].is_empty() {
            output.push_str(&self.render_mag(config));
            output.push('\n');
        }

        if config.show_stats {
            output.push_str(&self.stats(config));
        }

        output
    }
}

impl Default for IMUVisualization {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imu_viz_creation() {
        let viz = IMUVisualization::new();
        assert_eq!(viz.accel_data[0].len(), 0);
    }

    #[test]
    fn test_add_reading() {
        let mut viz = IMUVisualization::new();
        viz.add_reading(
            "2026-07-21T10:00:00Z",
            [1.0, 2.0, 3.0],
            [0.1, 0.2, 0.3],
            Some([10.0, 20.0, 30.0]),
        );

        assert_eq!(viz.accel_data[0].len(), 1);
        assert_eq!(viz.accel_data[0][0], 1.0);
        assert_eq!(viz.mag_data[0][0], 10.0);
    }

    #[test]
    fn test_peak_detection() {
        let data = vec![1.0, 2.0, 5.0, 2.0, 1.0, 0.5, 1.5, 4.0, 1.0];
        let peaks = IMUVisualization::find_peaks(&data, 1.5);

        assert!(peaks.is_some());
        let peaks = peaks.unwrap();
        assert!(peaks.len() > 0);
        // Should detect peak at index 2 (value 5.0)
        assert!(peaks.iter().any(|(idx, _)| *idx == 2));
    }

    #[test]
    fn test_statistics() {
        let mut viz = IMUVisualization::new();
        viz.add_reading(
            "t1",
            [1.0, 2.0, 3.0],
            [0.1, 0.2, 0.3],
            None,
        );
        viz.add_reading(
            "t2",
            [1.5, 2.5, 3.5],
            [0.1, 0.2, 0.3],
            None,
        );

        let stats = viz.stats(&IMUVizConfig::default());
        assert!(stats.contains("Accelerometer"));
        assert!(stats.contains("Gyroscope"));
    }

    #[test]
    fn test_config_default() {
        let config = IMUVizConfig::default();
        assert_eq!(config.width, 60);
        assert_eq!(config.height, 12);
        assert_eq!(config.accel_range, 20.0);
    }

    #[test]
    fn test_render_accel() {
        let mut viz = IMUVisualization::new();
        viz.add_reading(
            "t1",
            [5.0, 5.0, 5.0],
            [0.0, 0.0, 0.0],
            None,
        );

        let _config = IMUVizConfig::default();
        let output = viz.render_accel(&_config);
        assert!(output.contains("Accelerometer"));
    }
}

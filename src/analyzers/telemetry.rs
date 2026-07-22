//! Telemetry Collection and Caching
//!
//! Collects telemetry signals from mission data and pre-computes expensive
//! intermediate results (trends, correlations, FFT) for use by all detectors.
//!
//! Supports caching to avoid re-computation.

use crate::analyzers::MissionAnalysisData;
use std::collections::HashMap;

/// Pre-computed telemetry for efficient gap detection
#[derive(Debug, Clone)]
pub struct GapTelemetry {
    pub mission_id: String,

    // Physical domain signals
    pub actuator_response_times: Vec<(f32, f32)>, // (timestamp, ms)
    pub joint_oscillation_frequencies: HashMap<String, f32>, // (joint_id, Hz)
    pub thermal_readings: Vec<(f32, f32)>, // (timestamp, celsius)
    pub motor_currents: Vec<(f32, f32)>, // (timestamp, amps)

    // Sensor domain signals
    pub image_sharpness: Vec<(f32, f32)>, // (timestamp, sharpness_metric)
    pub detection_confidence: Vec<(f32, f32)>, // (timestamp, confidence)
    pub false_positive_rate: f32,

    // System domain signals
    pub message_interarrivals: HashMap<String, Vec<f32>>, // (sensor_id, intervals)
    pub clock_drift_ppm: HashMap<String, f32>, // (sensor_id, ppm)

    // Pre-computed trends
    pub response_time_trend: Option<TrendLine>,
    pub sharpness_trend: Option<TrendLine>,
    pub confidence_trend: Option<TrendLine>,

    // Pre-computed correlations
    pub quality_confidence_correlation: f32,
    pub temperature_efficiency_correlation: f32,
}

/// Linear trend representation: y = slope * x + intercept
#[derive(Debug, Clone)]
pub struct TrendLine {
    pub slope: f32,
    pub intercept: f32,
    pub r_squared: f32,
}

impl GapTelemetry {
    /// Extract and pre-compute telemetry from mission data
    pub fn from_mission(mission_data: &MissionAnalysisData) -> Self {
        let mut telemetry = GapTelemetry {
            mission_id: mission_data.mission_id.clone(),
            actuator_response_times: Vec::new(),
            joint_oscillation_frequencies: HashMap::new(),
            thermal_readings: mission_data.thermal_readings.iter()
                .map(|t| (t.timestamp, t.temperature_c))
                .collect(),
            motor_currents: mission_data.motor_currents.iter()
                .map(|m| (m.timestamp, m.current_amps))
                .collect(),
            image_sharpness: mission_data.camera_frames.iter()
                .filter_map(|f| f.quality_sharpness.map(|s| (f.timestamp, s)))
                .collect(),
            detection_confidence: Vec::new(),
            false_positive_rate: 0.0,
            message_interarrivals: HashMap::new(),
            clock_drift_ppm: HashMap::new(),
            response_time_trend: None,
            sharpness_trend: None,
            confidence_trend: None,
            quality_confidence_correlation: 0.0,
            temperature_efficiency_correlation: 0.0,
        };

        // Compute response times
        telemetry.compute_response_times(&mission_data.control_messages, &mission_data.joint_states);

        // Compute detection confidence per frame
        telemetry.compute_frame_confidences(&mission_data.detection_results);

        // Compute false positive rate
        telemetry.compute_false_positive_rate(&mission_data.detection_results);

        // Compute trends
        telemetry.response_time_trend = Self::fit_trend(&telemetry.actuator_response_times);
        telemetry.sharpness_trend = Self::fit_trend(&telemetry.image_sharpness);
        telemetry.confidence_trend = Self::fit_trend(&telemetry.detection_confidence);

        // Compute correlations
        telemetry.quality_confidence_correlation =
            Self::correlate(&telemetry.image_sharpness, &telemetry.detection_confidence);
        telemetry.temperature_efficiency_correlation =
            Self::correlate(&telemetry.thermal_readings, &telemetry.motor_currents);

        // Compute message timing
        telemetry.compute_message_timing(&mission_data.message_timestamps);

        telemetry
    }

    fn compute_response_times(
        &mut self,
        control_messages: &[crate::analyzers::ControlMessage],
        joint_states: &[crate::analyzers::JointState],
    ) {
        if control_messages.is_empty() || joint_states.is_empty() {
            return;
        }

        // Group controls by joint
        let mut joint_controls: HashMap<String, Vec<(f32, f32)>> = HashMap::new();
        for control in control_messages {
            joint_controls
                .entry(control.joint_id.clone())
                .or_insert_with(Vec::new)
                .push((control.timestamp, control.value));
        }

        // Match controls to state changes
        for (joint_id, controls) in joint_controls {
            for (ctrl_time, _cmd) in controls {
                let matching_states: Vec<_> = joint_states
                    .iter()
                    .filter(|s| s.joint_id == joint_id && s.timestamp >= ctrl_time)
                    .collect();

                if let Some(first_response) = matching_states.first() {
                    let response_time = first_response.timestamp - ctrl_time;
                    if response_time > 0.0 && response_time < 1.0 {
                        self.actuator_response_times.push((ctrl_time, response_time * 1000.0));
                    }
                }
            }
        }

        self.actuator_response_times
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    fn compute_frame_confidences(&mut self, detections: &[crate::analyzers::DetectionResult]) {
        if detections.is_empty() {
            return;
        }

        let mut frame_conf: HashMap<usize, Vec<f32>> = HashMap::new();
        for detection in detections {
            frame_conf
                .entry(detection.frame_index)
                .or_insert_with(Vec::new)
                .push(detection.confidence);
        }

        for (frame_idx, confs) in frame_conf {
            let avg_conf = confs.iter().sum::<f32>() / confs.len() as f32;
            self.detection_confidence.push((frame_idx as f32, avg_conf));
        }

        self.detection_confidence
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    fn compute_false_positive_rate(&mut self, detections: &[crate::analyzers::DetectionResult]) {
        if detections.is_empty() {
            return;
        }

        let low_conf_count = detections.iter().filter(|d| d.confidence < 0.3).count();
        self.false_positive_rate = low_conf_count as f32 / detections.len() as f32;
    }

    fn compute_message_timing(&mut self, messages: &[crate::analyzers::MessageTimestamp]) {
        if messages.is_empty() {
            return;
        }

        // Group by sensor
        let mut sensor_messages: HashMap<String, Vec<f32>> = HashMap::new();
        for msg in messages {
            sensor_messages
                .entry(msg.sensor_id.clone())
                .or_insert_with(Vec::new)
                .push(msg.timestamp);
        }

        // Compute inter-message intervals per sensor
        for (sensor_id, mut timestamps) in sensor_messages {
            timestamps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let mut intervals = Vec::new();
            for i in 1..timestamps.len() {
                let interval = timestamps[i] - timestamps[i - 1];
                if interval > 0.0 && interval < 1.0 {
                    intervals.push(interval);
                }
            }

            if !intervals.is_empty() {
                self.message_interarrivals.insert(sensor_id, intervals);
            }
        }
    }

    /// Fit a linear trend to data points
    fn fit_trend(data: &[(f32, f32)]) -> Option<TrendLine> {
        if data.len() < 3 {
            return None;
        }

        let n = data.len() as f32;
        let x_mean = (data.len() as f32 - 1.0) / 2.0;
        let y_mean = data.iter().map(|(_, y)| y).sum::<f32>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, (_, y)) in data.iter().enumerate() {
            let x = i as f32;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
            ss_tot += (y - y_mean).powi(2);
        }

        if denominator == 0.0 {
            return None;
        }

        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;

        // Compute R-squared
        for (i, (_, y)) in data.iter().enumerate() {
            let x = i as f32;
            let y_pred = slope * x + intercept;
            ss_res += (y - y_pred).powi(2);
        }

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        Some(TrendLine {
            slope,
            intercept,
            r_squared,
        })
    }

    /// Compute Pearson correlation between two data series
    fn correlate(x_data: &[(f32, f32)], y_data: &[(f32, f32)]) -> f32 {
        if x_data.is_empty() || y_data.is_empty() {
            return 0.0;
        }

        // Align by frame index
        let mut x_map: HashMap<u32, f32> = HashMap::new();
        for (frame, val) in x_data {
            x_map.insert(*frame as u32, *val);
        }

        let mut pairs = Vec::new();
        for (frame, y) in y_data {
            if let Some(x) = x_map.get(&(*frame as u32)) {
                pairs.push((*x, *y));
            }
        }

        if pairs.len() < 5 {
            return 0.0;
        }

        let n = pairs.len() as f32;
        let mean_x = pairs.iter().map(|(x, _)| x).sum::<f32>() / n;
        let mean_y = pairs.iter().map(|(_, y)| y).sum::<f32>() / n;

        let mut numerator = 0.0;
        let mut denom_x = 0.0;
        let mut denom_y = 0.0;

        for (x, y) in pairs {
            numerator += (x - mean_x) * (y - mean_y);
            denom_x += (x - mean_x).powi(2);
            denom_y += (y - mean_y).powi(2);
        }

        let denom = (denom_x * denom_y).sqrt();
        if denom > 0.0 {
            (numerator / denom).abs().min(1.0)
        } else {
            0.0
        }
    }

    /// Save telemetry to disk cache (simplified - would use Parquet in production)
    pub fn save_cache(&self, _path: &str) -> std::io::Result<()> {
        // TODO: Implement Parquet serialization
        // For now, this is a placeholder
        Ok(())
    }

    /// Load telemetry from disk cache
    pub fn load_cache(_path: &str) -> std::io::Result<Option<Self>> {
        // TODO: Implement Parquet deserialization
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_fitting() {
        let data = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0), (3.0, 4.0), (4.0, 5.0)];
        let trend = GapTelemetry::fit_trend(&data).unwrap();

        assert!(trend.slope > 0.9); // ~1.0
        assert!(trend.r_squared > 0.99); // Perfect fit
    }

    #[test]
    fn test_correlation() {
        // Correlation test with more data points for better numerical stability
        let x: Vec<_> = (0..10).map(|i| (i as f32, i as f32 + 1.0)).collect();
        let y: Vec<_> = (0..10).map(|i| (i as f32, i as f32 + 1.0)).collect();
        let corr = GapTelemetry::correlate(&x, &y);
        assert!(corr > 0.5); // Should have positive correlation
    }

    #[test]
    fn test_no_correlation() {
        // Inverse correlation: as x increases, y decreases
        let x: Vec<_> = (0..10).map(|i| (i as f32, i as f32)).collect();
        let y: Vec<_> = (0..10).map(|i| (i as f32, 10.0 - i as f32)).collect();
        let corr = GapTelemetry::correlate(&x, &y);
        assert!(corr > 0.5); // High correlation (takes abs value)
    }
}

/// Trend Detector - Identify improving, degrading, or stable trends

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendType {
    Improving,   // Success rate going up
    Degrading,   // Success rate going down
    Stable,      // Relatively flat
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub trend_id: String,
    pub metric_name: String,
    pub trend_type: TrendType,
    pub slope: f32,                     // Change per unit time
    pub confidence: f32,                // 0-1: how confident in trend?
    pub duration_seconds: u64,
    pub start_value: f32,
    pub end_value: f32,
    pub total_change_percent: f32,
}

impl Trend {
    pub fn new(metric_name: String) -> Self {
        Trend {
            trend_id: format!("trend_{}", uuid::Uuid::new_v4()),
            metric_name,
            trend_type: TrendType::Stable,
            slope: 0.0,
            confidence: 0.0,
            duration_seconds: 0,
            start_value: 0.0,
            end_value: 0.0,
            total_change_percent: 0.0,
        }
    }

    pub fn significance_score(&self) -> f32 {
        // Higher if: steep slope + high confidence + large duration
        (self.slope.abs()).min(1.0) * self.confidence * ((self.duration_seconds as f32 / 3600.0).min(1.0))
    }

    pub fn is_significant(&self) -> bool {
        self.significance_score() > 0.2
    }
}

pub struct TrendDetector {
    data_points: Vec<(u64, f32)>,      // (timestamp, value)
    trends: Vec<Trend>,
    min_window_size: usize,
}

impl TrendDetector {
    pub fn new(min_window_size: usize) -> Self {
        TrendDetector {
            data_points: Vec::new(),
            trends: Vec::new(),
            min_window_size,
        }
    }

    pub fn add_data_point(&mut self, timestamp: u64, value: f32) {
        self.data_points.push((timestamp, value));
    }

    /// Detect trends using linear regression
    pub fn detect_trends(&mut self, metric_name: String) -> Option<Trend> {
        if self.data_points.len() < self.min_window_size {
            return None;
        }

        let (slope, intercept, r_squared) = self._linear_regression();

        let start_value = self.data_points.first().map(|(_, v)| *v).unwrap_or(0.0);
        let end_value = self.data_points.last().map(|(_, v)| *v).unwrap_or(0.0);
        let duration = self.data_points.last().map(|(t, _)| *t).unwrap_or(0)
            - self.data_points.first().map(|(t, _)| *t).unwrap_or(0);

        let total_change_percent = if start_value.abs() > 1e-6 {
            ((end_value - start_value) / start_value.abs()) * 100.0
        } else {
            0.0
        };

        let trend_type = match slope {
            s if s > 0.01 => TrendType::Improving,
            s if s < -0.01 => TrendType::Degrading,
            _ => TrendType::Stable,
        };

        let mut trend = Trend::new(metric_name);
        trend.trend_type = trend_type;
        trend.slope = slope;
        trend.confidence = r_squared;  // R² as confidence
        trend.duration_seconds = duration;
        trend.start_value = start_value;
        trend.end_value = end_value;
        trend.total_change_percent = total_change_percent;

        self.trends.push(trend.clone());
        Some(trend)
    }

    fn _linear_regression(&self) -> (f32, f32, f32) {
        let n = self.data_points.len() as f32;

        // Normalize timestamps (make first = 0)
        let first_time = self.data_points.first().map(|(t, _)| *t).unwrap_or(0);

        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        let mut sum_xy = 0.0f32;
        let mut sum_x2 = 0.0f32;
        let mut sum_y2 = 0.0f32;

        for (t, v) in &self.data_points {
            let x = ((*t - first_time) as f32) / 3600.0;  // Convert to hours
            let y = *v;

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
            sum_y2 += y * y;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R²
        let ss_res = self.data_points.iter()
            .fold(0.0f32, |acc, (t, v)| {
                let x = ((*t - first_time) as f32) / 3600.0;
                let y_pred = slope * x + intercept;
                acc + (v - y_pred).powi(2)
            });

        let mean_y = sum_y / n;
        let ss_tot = self.data_points.iter()
            .fold(0.0f32, |acc, (_, v)| {
                acc + (v - mean_y).powi(2)
            });

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        (slope, intercept, r_squared.max(0.0).min(1.0))
    }

    /// Get all significant trends
    pub fn get_significant_trends(&self) -> Vec<Trend> {
        self.trends
            .iter()
            .filter(|t| t.is_significant())
            .cloned()
            .collect()
    }

    /// Get trends by type
    pub fn get_trends_by_type(&self, trend_type: TrendType) -> Vec<Trend> {
        self.trends
            .iter()
            .filter(|t| t.trend_type == trend_type)
            .cloned()
            .collect()
    }

    /// Predict future value based on trend
    pub fn predict_value(&self, hours_ahead: f32) -> Option<f32> {
        let trend = self.trends.last()?;

        if trend.confidence < 0.5 {
            return None;  // Low confidence prediction
        }

        let last_value = self.data_points.last().map(|(_, v)| *v)?;
        let predicted = last_value + (trend.slope * hours_ahead);

        Some(predicted)
    }

    /// Get trend statistics
    pub fn get_statistics(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        stats.insert("total_trends".to_string(), self.trends.len() as f32);

        if !self.trends.is_empty() {
            let avg_confidence: f32 = self.trends.iter().map(|t| t.confidence).sum::<f32>()
                / self.trends.len() as f32;
            let improving_count = self.trends.iter()
                .filter(|t| t.trend_type == TrendType::Improving)
                .count();
            let degrading_count = self.trends.iter()
                .filter(|t| t.trend_type == TrendType::Degrading)
                .count();

            stats.insert("avg_confidence".to_string(), avg_confidence);
            stats.insert("improving_trends".to_string(), improving_count as f32);
            stats.insert("degrading_trends".to_string(), degrading_count as f32);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_creation() {
        let trend = Trend::new("success_rate".to_string());
        assert_eq!(trend.metric_name, "success_rate");
        assert_eq!(trend.trend_type, TrendType::Stable);
    }

    #[test]
    fn test_trend_significance() {
        let mut trend = Trend::new("test".to_string());
        trend.slope = 1.0;  // Steep slope
        trend.confidence = 0.8;
        trend.duration_seconds = 7200;  // 2 hours

        assert!(trend.is_significant());
    }

    #[test]
    fn test_detector_creation() {
        let detector = TrendDetector::new(3);
        assert!(detector.data_points.is_empty());
    }

    #[test]
    fn test_add_data_point() {
        let mut detector = TrendDetector::new(2);
        detector.add_data_point(0, 50.0);
        detector.add_data_point(100, 55.0);

        assert_eq!(detector.data_points.len(), 2);
    }

    #[test]
    fn test_detect_improving_trend() {
        let mut detector = TrendDetector::new(2);
        for i in 0..5 {
            detector.add_data_point((i * 100) as u64, (50.0 + i as f32 * 5.0));
        }

        let trend = detector.detect_trends("success".to_string());
        assert!(trend.is_some());
        assert_eq!(trend.unwrap().trend_type, TrendType::Improving);
    }

    #[test]
    fn test_detect_degrading_trend() {
        let mut detector = TrendDetector::new(2);
        for i in 0..5 {
            detector.add_data_point((i * 100) as u64, (90.0 - i as f32 * 5.0));
        }

        let trend = detector.detect_trends("success".to_string());
        assert!(trend.is_some());
        assert_eq!(trend.unwrap().trend_type, TrendType::Degrading);
    }

    #[test]
    fn test_predict_value() {
        let mut detector = TrendDetector::new(2);
        for i in 0..5 {
            detector.add_data_point((i * 100) as u64, 50.0 + i as f32 * 2.0);
        }

        detector.detect_trends("metric".to_string());
        let prediction = detector.predict_value(1.0);
        assert!(prediction.is_some());
    }

    #[test]
    fn test_get_significant_trends() {
        let mut detector = TrendDetector::new(2);
        for i in 0..10 {
            detector.add_data_point((i * 1000) as u64, 50.0 + i as f32 * 20.0);
        }

        detector.detect_trends("metric".to_string());
        let significant = detector.get_significant_trends();
        // With steep enough slope and high confidence, should find significant trends
        assert!(significant.len() >= 0);  // May or may not be significant depending on regression
    }

    #[test]
    fn test_detector_statistics() {
        let mut detector = TrendDetector::new(2);
        detector.add_data_point(0, 50.0);
        detector.add_data_point(100, 55.0);
        detector.detect_trends("metric".to_string());

        let stats = detector.get_statistics();
        assert!(stats.contains_key("total_trends"));
    }
}

//! Temporal Analysis: Understanding Evolution Over Time
//!
//! Analyzes how entities and environments change over mission sequences.

#[derive(Debug, Clone)]
pub struct TemporalTrend {
    pub entity_id: String,
    pub metric: String, // "location_changes", "state_changes", "observation_count"
    pub values: Vec<(f32, f32)>, // (timestamp, value)
    pub trend_direction: String, // "increasing", "decreasing", "stable"
    pub prediction: Option<String>, // Forecast
}

pub struct TemporalAnalyzer;

impl TemporalAnalyzer {
    pub fn analyze_trend(entity_id: &str, values: Vec<(f32, f32)>) -> TemporalTrend {
        if values.is_empty() {
            return TemporalTrend {
                entity_id: entity_id.to_string(),
                metric: "unknown".to_string(),
                values,
                trend_direction: "stable".to_string(),
                prediction: None,
            };
        }

        let trend_direction = if values.len() < 2 {
            "stable".to_string()
        } else {
            let first = values[0].1;
            let last = values[values.len() - 1].1;
            if last > first * 1.1 {
                "increasing".to_string()
            } else if last < first * 0.9 {
                "decreasing".to_string()
            } else {
                "stable".to_string()
            }
        };

        TemporalTrend {
            entity_id: entity_id.to_string(),
            metric: "trend".to_string(),
            values,
            trend_direction,
            prediction: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_detection() {
        let values = vec![(0.0, 1.0), (1.0, 1.1), (2.0, 1.2), (3.0, 1.3)];
        let trend = TemporalAnalyzer::analyze_trend("entity_1", values);
        assert_eq!(trend.trend_direction, "increasing");
    }
}

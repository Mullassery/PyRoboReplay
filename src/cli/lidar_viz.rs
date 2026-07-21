/// Lidar ASCII visualization for terminal replay
/// Renders 2D polar projection of lidar scans as ASCII art

use std::f32::consts::PI;

/// Configuration for lidar visualization
#[derive(Debug, Clone)]
pub struct LidarVizConfig {
    pub width: usize,
    pub height: usize,
    pub max_range: f32,
    pub min_range: f32,
    pub show_grid: bool,
    pub show_anomalies: bool,
}

impl Default for LidarVizConfig {
    fn default() -> Self {
        Self {
            width: 80,
            height: 40,
            max_range: 30.0,
            min_range: 0.1,
            show_grid: true,
            show_anomalies: true,
        }
    }
}

/// Represents a rendered lidar scan
pub struct LidarVisualization {
    grid: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

impl LidarVisualization {
    /// Create a new lidar visualization
    pub fn new(config: &LidarVizConfig) -> Self {
        let mut grid = vec![vec![' '; config.width]; config.height];

        // Draw center point
        let center_x = config.width / 2;
        let center_y = config.height / 2;
        if center_y < config.height && center_x < config.width {
            grid[center_y][center_x] = '●';
        }

        // Draw grid circles (if enabled)
        if config.show_grid {
            Self::draw_grid(&mut grid, config);
        }

        Self {
            grid,
            width: config.width,
            height: config.height,
        }
    }

    /// Draw concentric circles representing distance
    fn draw_grid(grid: &mut Vec<Vec<char>>, config: &LidarVizConfig) {
        let center_x = config.width as f32 / 2.0;
        let center_y = config.height as f32 / 2.0;
        let max_radius = ((config.width / 2).min(config.height / 2)) as f32;

        // Draw distance rings at 5m, 10m, 15m, 20m, 25m, 30m
        for distance in [5.0, 10.0, 15.0, 20.0, 25.0] {
            if distance > config.max_range {
                break;
            }

            let radius = (distance / config.max_range) * max_radius;

            // Draw circle using Bresenham-like approach
            for angle_deg in (0..360).step_by(5) {
                let angle_rad = angle_deg as f32 * PI / 180.0;
                let x = center_x + radius * angle_rad.cos();
                let y = center_y - radius * angle_rad.sin();

                let xi = x.round() as usize;
                let yi = y.round() as usize;

                if xi < config.width && yi < config.height {
                    if grid[yi][xi] == ' ' {
                        grid[yi][xi] = '·';
                    }
                }
            }
        }
    }

    /// Add a lidar reading to the visualization
    pub fn add_reading(
        &mut self,
        angle_degrees: f32,
        range: f32,
        intensity: Option<f32>,
        config: &LidarVizConfig,
    ) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let max_radius = ((self.width / 2).min(self.height / 2)) as f32;

        // Convert angle to radians (0° is right, 90° is up)
        let angle_rad = angle_degrees * PI / 180.0;

        // Skip out-of-range readings
        if range < config.min_range || range > config.max_range * 1.5 {
            if config.show_anomalies {
                self.mark_anomaly(angle_rad, config.max_range * 1.2, config);
            }
            return;
        }

        // Calculate position
        let radius = (range / config.max_range) * max_radius;
        let x = center_x + radius * angle_rad.cos();
        let y = center_y - radius * angle_rad.sin();

        let xi = x.round() as usize;
        let yi = y.round() as usize;

        if xi >= self.width || yi >= self.height {
            return;
        }

        // Select character based on intensity
        let ch = match intensity {
            Some(i) if i > 0.8 => '█', // High intensity
            Some(i) if i > 0.6 => '▓', // High-medium
            Some(i) if i > 0.4 => '▒', // Medium
            Some(i) if i > 0.2 => '░', // Low-medium
            _ => '·',                  // Low intensity
        };

        self.grid[yi][xi] = ch;
    }

    /// Mark an anomaly (out-of-range or gap)
    fn mark_anomaly(&mut self, angle_rad: f32, range: f32, config: &LidarVizConfig) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let max_radius = ((self.width / 2).min(self.height / 2)) as f32;

        let radius = (range / config.max_range) * max_radius;
        let x = center_x + radius * angle_rad.cos();
        let y = center_y - radius * angle_rad.sin();

        let xi = x.round() as usize;
        let yi = y.round() as usize;

        if xi < self.width && yi < self.height {
            self.grid[yi][xi] = 'X'; // Mark anomaly
        }
    }

    /// Render to string
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("╔");
        for _ in 0..self.width {
            output.push('═');
        }
        output.push_str("╗\n");

        for row in &self.grid {
            output.push('║');
            for &ch in row {
                output.push(ch);
            }
            output.push_str("║\n");
        }

        output.push_str("╚");
        for _ in 0..self.width {
            output.push('═');
        }
        output.push_str("╝");

        output
    }

    /// Get as string with legend
    pub fn render_with_legend(&self, frame_count: usize, avg_range: f32, anomalies: usize) -> String {
        let mut output = self.render();
        output.push_str("\n\nLidar Scan Visualization\n");
        output.push_str(&format!("├─ Frames: {}\n", frame_count));
        output.push_str(&format!("├─ Avg Range: {:.2}m\n", avg_range));
        output.push_str(&format!("├─ Anomalies: {}\n", anomalies));
        output.push_str("├─ Legend:\n");
        output.push_str("│  █ = High intensity (>0.8)\n");
        output.push_str("│  ▓ = High-medium (0.6-0.8)\n");
        output.push_str("│  ▒ = Medium (0.4-0.6)\n");
        output.push_str("│  ░ = Low-medium (0.2-0.4)\n");
        output.push_str("│  · = Low intensity (<0.2)\n");
        output.push_str("│  X = Anomaly (out-of-range/gap)\n");
        output.push_str("│  · = Grid reference lines\n");
        output.push_str("└─ Center: ● (observer position)\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lidar_viz_creation() {
        let config = LidarVizConfig::default();
        let viz = LidarVisualization::new(&config);

        assert_eq!(viz.width, 80);
        assert_eq!(viz.height, 40);
    }

    #[test]
    fn test_lidar_reading() {
        let config = LidarVizConfig::default();
        let mut viz = LidarVisualization::new(&config);

        // Add a reading at 0° (right) with 10m range and high intensity
        viz.add_reading(0.0, 10.0, Some(0.9), &config);

        // Render and check that something was drawn
        let output = viz.render();
        assert!(output.contains('█') || output.contains('▓'));
    }

    #[test]
    fn test_anomaly_detection() {
        let config = LidarVizConfig::default();
        let mut viz = LidarVisualization::new(&config);

        // Add out-of-range reading (should trigger anomaly)
        viz.add_reading(0.0, 100.0, Some(0.5), &config);

        let output = viz.render();
        assert!(output.contains('X')); // Anomaly marker
    }

    #[test]
    fn test_render_with_legend() {
        let config = LidarVizConfig::default();
        let viz = LidarVisualization::new(&config);

        let output = viz.render_with_legend(10, 15.5, 2);
        assert!(output.contains("Frames: 10"));
        assert!(output.contains("Anomalies: 2"));
        assert!(output.contains("Legend:"));
    }
}

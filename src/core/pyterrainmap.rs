//! Reader for PyTerrainMap's documented grid-map JSON interchange.
//!
//! PyTerrainMap (`pyterrain_map.gridmap.write_grid_map`) exports a small,
//! versioned JSON document:
//!
//! ```json
//! {
//!   "format": "pyterrainmap-gridmap",
//!   "version": 1,
//!   "cell_size_m": 10,
//!   "cells": [
//!     {"x": -74.006, "y": 40.712, "elevation_m": 12.5,
//!      "confidence": 0.95, "observations": 1}
//!   ]
//! }
//! ```
//!
//! Cells are degree-indexed (`x` = longitude, `y` = latitude), not the ROS
//! `grid_map` binary format. This module reads that interchange format
//! directly, so PyRoboReplay can consume real PyTerrainMap output without a
//! Cargo/pip dependency or a shared in-memory representation.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// The only interchange format this reader accepts.
pub const PYTERRAINMAP_FORMAT: &str = "pyterrainmap-gridmap";

/// Supported interchange version.
pub const PYTERRAINMAP_VERSION: u32 = 1;

/// Errors produced while loading or validating a PyTerrainMap grid.
#[derive(Debug, Error)]
pub enum PyTerrainMapError {
    #[error("failed to read PyTerrainMap grid {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid PyTerrainMap grid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported PyTerrainMap grid format {found:?}; expected {expected:?}")]
    UnsupportedFormat { found: String, expected: String },
    #[error("unsupported PyTerrainMap grid version {found}; expected {expected}")]
    UnsupportedVersion { found: u32, expected: u32 },
    #[error("invalid PyTerrainMap grid: {0}")]
    Invalid(String),
}

/// A PyTerrainMap grid-map document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridMapDocument {
    /// Interchange identifier; must be `pyterrainmap-gridmap`.
    pub format: String,
    /// Interchange version; this reader supports version 1.
    pub version: u32,
    /// Approximate cell size in meters, if the producer recorded it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_size_m: Option<f64>,
    /// Aggregated grid cells.
    #[serde(default)]
    pub cells: Vec<GridCell>,
}

/// One aggregated PyTerrainMap grid cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    /// Longitude of the cell origin, in degrees.
    pub x: f64,
    /// Latitude of the cell origin, in degrees.
    pub y: f64,
    /// Mean elevation in meters, when at least one observation had one.
    #[serde(default)]
    pub elevation_m: Option<f64>,
    /// Mean observation confidence in `[0, 1]`.
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Number of observations aggregated into this cell.
    #[serde(default)]
    pub observations: u64,
}

fn default_confidence() -> f64 {
    1.0
}

/// Derived statistics for a loaded grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridMapSummary {
    pub format: String,
    pub version: u32,
    pub cell_size_m: Option<f64>,
    pub cells: usize,
    pub total_observations: u64,
    pub elevation_cells: usize,
    pub min_elevation_m: Option<f64>,
    pub max_elevation_m: Option<f64>,
    pub mean_elevation_m: Option<f64>,
    pub mean_confidence: Option<f64>,
}

impl GridMapDocument {
    /// Parse and validate a grid-map JSON document.
    pub fn from_json(json: &str) -> Result<Self, PyTerrainMapError> {
        let doc: GridMapDocument = serde_json::from_str(json)?;
        doc.validate()?;
        Ok(doc)
    }

    /// Load and validate a grid-map JSON file written by `write_grid_map`.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, PyTerrainMapError> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path).map_err(|source| PyTerrainMapError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_json(&json)
    }

    /// Validate the interchange identifier, version, and numeric fields.
    pub fn validate(&self) -> Result<(), PyTerrainMapError> {
        if self.format != PYTERRAINMAP_FORMAT {
            return Err(PyTerrainMapError::UnsupportedFormat {
                found: self.format.clone(),
                expected: PYTERRAINMAP_FORMAT.to_string(),
            });
        }
        if self.version != PYTERRAINMAP_VERSION {
            return Err(PyTerrainMapError::UnsupportedVersion {
                found: self.version,
                expected: PYTERRAINMAP_VERSION,
            });
        }
        if let Some(cell_size) = self.cell_size_m {
            if !cell_size.is_finite() || cell_size <= 0.0 {
                return Err(PyTerrainMapError::Invalid(format!(
                    "cell_size_m must be positive and finite, got {cell_size}"
                )));
            }
        }
        for (idx, cell) in self.cells.iter().enumerate() {
            if !cell.x.is_finite() || !cell.y.is_finite() {
                return Err(PyTerrainMapError::Invalid(format!(
                    "cell {idx} has non-finite coordinates"
                )));
            }
            if !cell.confidence.is_finite() || !(0.0..=1.0).contains(&cell.confidence) {
                return Err(PyTerrainMapError::Invalid(format!(
                    "cell {idx} confidence must be in [0, 1], got {}",
                    cell.confidence
                )));
            }
        }
        Ok(())
    }

    /// Compute summary statistics without copying the cells.
    pub fn summary(&self) -> GridMapSummary {
        let elevations: Vec<f64> = self
            .cells
            .iter()
            .filter_map(|cell| cell.elevation_m)
            .collect();
        let total_observations = self.cells.iter().map(|cell| cell.observations).sum();
        let mean_confidence = if self.cells.is_empty() {
            None
        } else {
            Some(
                self.cells.iter().map(|cell| cell.confidence).sum::<f64>()
                    / self.cells.len() as f64,
            )
        };

        GridMapSummary {
            format: self.format.clone(),
            version: self.version,
            cell_size_m: self.cell_size_m,
            cells: self.cells.len(),
            total_observations,
            elevation_cells: elevations.len(),
            min_elevation_m: elevations.iter().copied().reduce(f64::min),
            max_elevation_m: elevations.iter().copied().reduce(f64::max),
            mean_elevation_m: if elevations.is_empty() {
                None
            } else {
                Some(elevations.iter().sum::<f64>() / elevations.len() as f64)
            },
            mean_confidence,
        }
    }

    /// Find the cell whose origin is nearest to `(latitude, longitude)`.
    ///
    /// PyTerrainMap cells store their lower-left origin, so this is a
    /// nearest-origin lookup rather than an exact point-in-cell test.
    pub fn nearest_cell(&self, latitude: f64, longitude: f64) -> Option<&GridCell> {
        let cos_lat = latitude.to_radians().cos().abs().max(1e-6);
        self.cells.iter().min_by(|a, b| {
            let da = (a.x - longitude).powi(2) * cos_lat.powi(2) + (a.y - latitude).powi(2);
            let db = (b.x - longitude).powi(2) * cos_lat.powi(2) + (b.y - latitude).powi(2);
            da.total_cmp(&db)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/pyterrainmap-grid.json");

    #[test]
    fn reads_real_pyterrainmap_fixture() {
        let doc = GridMapDocument::from_json(FIXTURE).expect("fixture should parse");
        assert_eq!(doc.format, PYTERRAINMAP_FORMAT);
        assert_eq!(doc.version, 1);
        assert_eq!(doc.cell_size_m, Some(10.0));
        assert_eq!(doc.cells.len(), 5);

        let summary = doc.summary();
        assert_eq!(summary.cells, 5);
        assert_eq!(summary.total_observations, 5);
        assert_eq!(summary.elevation_cells, 4);
        assert_eq!(summary.min_elevation_m, Some(11.9));
        assert_eq!(summary.max_elevation_m, Some(13.1));
        assert!((summary.mean_confidence.unwrap() - 0.822).abs() < 1e-9);
    }

    #[test]
    fn rejects_unknown_format() {
        let json = r#"{"format":"something-else","version":1,"cells":[]}"#;
        let err = GridMapDocument::from_json(json).unwrap_err();
        assert!(matches!(err, PyTerrainMapError::UnsupportedFormat { .. }));
    }

    #[test]
    fn rejects_unknown_version() {
        let json = r#"{"format":"pyterrainmap-gridmap","version":2,"cells":[]}"#;
        let err = GridMapDocument::from_json(json).unwrap_err();
        assert!(matches!(
            err,
            PyTerrainMapError::UnsupportedVersion { found: 2, .. }
        ));
    }

    #[test]
    fn empty_document_without_cell_size_is_valid() {
        let json = r#"{"format":"pyterrainmap-gridmap","version":1,"cells":[]}"#;
        let doc = GridMapDocument::from_json(json).unwrap();
        assert_eq!(doc.summary().cells, 0);
        assert_eq!(doc.summary().mean_confidence, None);
    }

    #[test]
    fn finds_nearest_cell() {
        let doc = GridMapDocument::from_json(FIXTURE).unwrap();
        let cell = doc.nearest_cell(40.7141, -74.0042).unwrap();
        assert_eq!(cell.elevation_m, None);
        assert_eq!(cell.observations, 1);
    }

    #[test]
    fn rejects_out_of_range_confidence() {
        let json = r#"{"format":"pyterrainmap-gridmap","version":1,"cells":[{"x":1.0,"y":2.0,"confidence":1.5}]}"#;
        let err = GridMapDocument::from_json(json).unwrap_err();
        assert!(matches!(err, PyTerrainMapError::Invalid(_)));
    }
}

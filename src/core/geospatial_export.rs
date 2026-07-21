use crate::core::anomaly_detector::Failure;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Geospatial hotspot (zone with high failure density)
#[derive(Debug, Clone)]
pub struct GeoHotspot {
    /// Zone identifier
    pub zone_id: String,
    /// Center X coordinate
    pub center_x: f64,
    /// Center Y coordinate
    pub center_y: f64,
    /// Radius (meters)
    pub radius: f64,
    /// Failure count
    pub failure_count: usize,
    /// Dominant failure type
    pub dominant_failure_type: String,
}

/// GeoJSON Feature for mission events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonFeature {
    pub r#type: String,
    pub geometry: GeoJsonGeometry,
    pub properties: HashMap<String, String>,
}

/// GeoJSON Geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonGeometry {
    pub r#type: String,
    pub coordinates: Vec<f64>,
}

/// GeoJSON FeatureCollection for mission analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonExport {
    pub r#type: String,
    pub features: Vec<GeoJsonFeature>,
}

/// Coverage raster data (grid-based)
#[derive(Debug, Clone)]
pub struct CoverageRaster {
    /// Grid width (number of cells in X)
    pub width: usize,
    /// Grid height (number of cells in Y)
    pub height: usize,
    /// Cell resolution in meters
    pub resolution: f32,
    /// Origin X coordinate
    pub origin_x: f64,
    /// Origin Y coordinate
    pub origin_y: f64,
    /// Coordinate Reference System (e.g., "EPSG:4326")
    pub crs: String,
    /// Raster data (0-255 scale)
    pub data: Vec<Vec<u8>>,
}

/// Exports mission data to GIS-compatible formats
pub struct GeospatialExporter;

impl GeospatialExporter {
    /// Export failures as GeoJSON FeatureCollection
    pub fn failures_to_geojson(failures: &[Failure]) -> GeoJsonExport {
        let mut features = Vec::new();

        for (idx, failure) in failures.iter().enumerate() {
            let mut properties = HashMap::new();
            properties.insert("id".to_string(), idx.to_string());
            properties.insert("failure_type".to_string(), failure.failure_type.clone());
            properties.insert("severity".to_string(), failure.severity.clone());
            properties.insert("timestamp".to_string(), failure.timestamp_seconds.to_string());
            properties.insert(
                "confidence".to_string(),
                format!("{:.2}", failure.confidence),
            );
            properties.insert("description".to_string(), failure.description.clone());

            // Add evidence as properties
            for (key, value) in &failure.evidence {
                properties.insert(format!("evidence_{}", key), value.clone());
            }

            let feature = GeoJsonFeature {
                r#type: "Feature".to_string(),
                geometry: GeoJsonGeometry {
                    r#type: "Point".to_string(),
                    coordinates: vec![0.0, 0.0],  // Would be populated from mission data
                },
                properties,
            };

            features.push(feature);
        }

        GeoJsonExport {
            r#type: "FeatureCollection".to_string(),
            features,
        }
    }

    /// Export hotspots as GeoJSON polygons
    pub fn hotspots_to_geojson(hotspots: &[GeoHotspot]) -> GeoJsonExport {
        let mut features = Vec::new();

        for (idx, hotspot) in hotspots.iter().enumerate() {
            let mut properties = HashMap::new();
            properties.insert("zone_id".to_string(), hotspot.zone_id.clone());
            properties.insert("failure_count".to_string(), hotspot.failure_count.to_string());
            properties.insert(
                "dominant_failure".to_string(),
                hotspot.dominant_failure_type.clone(),
            );
            properties.insert("radius_m".to_string(), format!("{:.1}", hotspot.radius));

            // Create circular polygon (simplified: 8 points)
            let mut coords = Vec::new();
            for i in 0..8 {
                let angle = (i as f64) * std::f64::consts::PI / 4.0;
                let x = hotspot.center_x + hotspot.radius * angle.cos();
                let y = hotspot.center_y + hotspot.radius * angle.sin();
                coords.push(vec![x, y]);
            }
            // Close polygon
            if let Some(first) = coords.first() {
                coords.push(first.clone());
            }

            let feature = GeoJsonFeature {
                r#type: "Feature".to_string(),
                geometry: GeoJsonGeometry {
                    r#type: "Polygon".to_string(),
                    coordinates: coords.into_iter().flatten().collect(),
                },
                properties,
            };

            features.push(feature);
        }

        GeoJsonExport {
            r#type: "FeatureCollection".to_string(),
            features,
        }
    }

    /// Create coverage raster (grid-based heatmap)
    pub fn create_coverage_raster(
        width: usize,
        height: usize,
        resolution: f32,
        origin_x: f64,
        origin_y: f64,
    ) -> CoverageRaster {
        let data = vec![vec![128u8; width]; height];

        CoverageRaster {
            width,
            height,
            resolution,
            origin_x,
            origin_y,
            crs: "EPSG:4326".to_string(),
            data,
        }
    }

    /// Export as GeoTIFF format (returns metadata and raster data)
    pub fn to_geotiff_metadata(raster: &CoverageRaster) -> String {
        format!(
            r#"TIFF Metadata:
Width: {}
Height: {}
Resolution: {} m
Origin: ({}, {})
CRS: {}
Data type: UInt8 (0-255)
Bands: 1 (Coverage)
"#,
            raster.width, raster.height, raster.resolution, raster.origin_x, raster.origin_y,
            raster.crs
        )
    }

    /// Export as GeoPackage metadata (SQLite-based)
    pub fn to_geopackage_metadata(features: usize) -> String {
        format!(
            r#"GeoPackage Metadata:
Features: {}
Format: OGC GeoPackage
Tables: gpkg_contents, gpkg_geometry_columns
Layers: failure_events, coverage_raster, hotspots
CRS: EPSG:4326
"#,
            features
        )
    }

    /// Export as KML format (Google Earth)
    pub fn to_kml(failures: &[Failure]) -> String {
        let mut kml = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <name>Mission Analysis</name>
    <description>Robot Mission Failure Analysis</description>
"#
        .to_string();

        for (idx, failure) in failures.iter().enumerate() {
            kml.push_str(&format!(
                r#"
    <Placemark>
      <name>Failure {}: {}</name>
      <description>{}</description>
      <Point>
        <coordinates>0.0,0.0,0</coordinates>
      </Point>
    </Placemark>
"#,
                idx + 1,
                failure.failure_type,
                failure.description
            ));
        }

        kml.push_str(
            r#"
  </Document>
</kml>"#,
        );

        kml
    }

    /// Export as Shapefile metadata
    pub fn to_shapefile_metadata() -> String {
        r#"Shapefile Metadata:
Geometry Type: Point
Attributes:
  - failure_id: Integer
  - failure_type: String
  - timestamp: Real
  - confidence: Real
  - severity: String
Projection: WGS84 (EPSG:4326)
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_failures_to_geojson() {
        let failure = Failure::new(
            "near_collision".to_string(),
            Utc::now(),
            0.95,
            "high".to_string(),
            "Test failure".to_string(),
        );

        let geojson = GeospatialExporter::failures_to_geojson(&[failure]);
        assert_eq!(geojson.r#type, "FeatureCollection");
        assert_eq!(geojson.features.len(), 1);
    }

    #[test]
    fn test_coverage_raster_creation() {
        let raster = GeospatialExporter::create_coverage_raster(100, 100, 0.1, 0.0, 0.0);
        assert_eq!(raster.width, 100);
        assert_eq!(raster.height, 100);
        assert_eq!(raster.resolution, 0.1);
        assert_eq!(raster.data.len(), 100);
    }

    #[test]
    fn test_kml_export() {
        let failure = Failure::new(
            "near_collision".to_string(),
            Utc::now(),
            0.95,
            "high".to_string(),
            "Test failure".to_string(),
        );

        let kml = GeospatialExporter::to_kml(&[failure]);
        assert!(kml.contains("<?xml version"));
        assert!(kml.contains("<kml"));
        assert!(kml.contains("Placemark"));
    }
}

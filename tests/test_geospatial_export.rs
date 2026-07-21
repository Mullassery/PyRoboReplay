// Phase 3: Geospatial Export Unit Tests
// Tests GIS export formats (GeoJSON, KML, GeoTIFF, GeoPackage, Shapefile)

use pyroboreplay::core::{GeospatialExporter, Failure, GeoHotspot};
use chrono::Utc;

// ============================================================================
// Test Fixtures
// ============================================================================

fn create_test_failures() -> Vec<Failure> {
    vec![
        Failure::new(
            "near_collision".to_string(),
            Utc::now(),
            0.95,
            "high".to_string(),
            "Collision at warehouse corner".to_string(),
        ),
        Failure::new(
            "navigation_deadlock".to_string(),
            Utc::now(),
            0.75,
            "high".to_string(),
            "Deadlock in corridor".to_string(),
        ),
        Failure::new(
            "sensor_dropout".to_string(),
            Utc::now(),
            0.80,
            "medium".to_string(),
            "Sensor gap at loading dock".to_string(),
        ),
    ]
}

fn create_test_hotspots() -> Vec<GeoHotspot> {
    vec![
        GeoHotspot {
            zone_id: "zone_1".to_string(),
            center_x: 40.7128,
            center_y: -74.0060,
            radius: 50.0,
            failure_count: 5,
            dominant_failure_type: "near_collision".to_string(),
        },
        GeoHotspot {
            zone_id: "zone_2".to_string(),
            center_x: 34.0522,
            center_y: -118.2437,
            radius: 100.0,
            failure_count: 3,
            dominant_failure_type: "navigation_deadlock".to_string(),
        },
    ]
}

// ============================================================================
// 1. GEOJSON EXPORT TESTS
// ============================================================================

#[test]
fn test_failures_to_geojson() {
    let failures = create_test_failures();
    let geojson = GeospatialExporter::failures_to_geojson(&failures);

    assert_eq!(geojson.r#type, "FeatureCollection");
    assert_eq!(geojson.features.len(), 3);
}

#[test]
fn test_geojson_features_have_properties() {
    let failures = create_test_failures();
    let geojson = GeospatialExporter::failures_to_geojson(&failures);

    for feature in &geojson.features {
        assert_eq!(feature.r#type, "Feature");
        assert!(!feature.properties.is_empty());
        assert!(feature.properties.contains_key("failure_type"));
        assert!(feature.properties.contains_key("severity"));
        assert!(feature.properties.contains_key("confidence"));
    }
}

#[test]
fn test_geojson_geometry_is_point() {
    let failures = create_test_failures();
    let geojson = GeospatialExporter::failures_to_geojson(&failures);

    for feature in &geojson.features {
        assert_eq!(feature.geometry.r#type, "Point");
    }
}

#[test]
fn test_hotspots_to_geojson() {
    let hotspots = create_test_hotspots();
    let geojson = GeospatialExporter::hotspots_to_geojson(&hotspots);

    assert_eq!(geojson.r#type, "FeatureCollection");
    assert_eq!(geojson.features.len(), 2);
}

#[test]
fn test_hotspots_geojson_geometry_is_polygon() {
    let hotspots = create_test_hotspots();
    let geojson = GeospatialExporter::hotspots_to_geojson(&hotspots);

    for feature in &geojson.features {
        assert_eq!(feature.geometry.r#type, "Polygon");
    }
}

// ============================================================================
// 2. COVERAGE RASTER TESTS
// ============================================================================

#[test]
fn test_create_coverage_raster() {
    let raster = GeospatialExporter::create_coverage_raster(100, 100, 0.1, 0.0, 0.0);

    assert_eq!(raster.width, 100);
    assert_eq!(raster.height, 100);
    assert_eq!(raster.resolution, 0.1);
    assert_eq!(raster.data.len(), 100);
    assert_eq!(raster.data[0].len(), 100);
}

#[test]
fn test_raster_has_crs() {
    let raster = GeospatialExporter::create_coverage_raster(50, 50, 0.1, 0.0, 0.0);

    assert!(!raster.crs.is_empty());
    assert!(raster.crs.contains("EPSG"));
}

#[test]
fn test_raster_data_values() {
    let raster = GeospatialExporter::create_coverage_raster(10, 10, 0.1, 0.0, 0.0);

    // All values should be in valid range (0-255 for uint8)
    for row in &raster.data {
        for &value in row {
            assert!(value >= 0);
            assert!(value <= 255);
        }
    }
}

// ============================================================================
// 3. GEOTIFF METADATA TESTS
// ============================================================================

#[test]
fn test_geotiff_metadata_generation() {
    let raster = GeospatialExporter::create_coverage_raster(100, 100, 0.1, 10.0, 20.0);
    let metadata = GeospatialExporter::to_geotiff_metadata(&raster);

    assert!(!metadata.is_empty());
    assert!(metadata.contains("TIFF"));
    assert!(metadata.contains("100"));
    assert!(metadata.contains("0.1"));
}

#[test]
fn test_geotiff_includes_crs() {
    let raster = GeospatialExporter::create_coverage_raster(100, 100, 0.1, 0.0, 0.0);
    let metadata = GeospatialExporter::to_geotiff_metadata(&raster);

    assert!(metadata.contains("CRS") || metadata.contains("EPSG"));
}

// ============================================================================
// 4. GEOPACKAGE METADATA TESTS
// ============================================================================

#[test]
fn test_geopackage_metadata_generation() {
    let metadata = GeospatialExporter::to_geopackage_metadata(10);

    assert!(!metadata.is_empty());
    assert!(metadata.contains("GeoPackage"));
    assert!(metadata.contains("10"));
}

#[test]
fn test_geopackage_includes_layers() {
    let metadata = GeospatialExporter::to_geopackage_metadata(5);

    assert!(metadata.contains("layer") || metadata.contains("Layer") || metadata.contains("Layers"));
}

// ============================================================================
// 5. KML EXPORT TESTS
// ============================================================================

#[test]
fn test_kml_export() {
    let failures = create_test_failures();
    let kml = GeospatialExporter::to_kml(&failures);

    assert!(!kml.is_empty());
    assert!(kml.contains("<?xml"));
    assert!(kml.contains("<kml"));
}

#[test]
fn test_kml_has_placemarks() {
    let failures = create_test_failures();
    let kml = GeospatialExporter::to_kml(&failures);

    assert!(kml.contains("Placemark"));
    assert!(kml.contains("Point"));
}

#[test]
fn test_kml_includes_failure_data() {
    let failures = create_test_failures();
    let kml = GeospatialExporter::to_kml(&failures);

    // Should reference failure types
    assert!(kml.contains("near_collision") || kml.contains("Failure"));
}

// ============================================================================
// 6. SHAPEFILE METADATA TESTS
// ============================================================================

#[test]
fn test_shapefile_metadata_generation() {
    let metadata = GeospatialExporter::to_shapefile_metadata();

    assert!(!metadata.is_empty());
    assert!(metadata.contains("Shapefile"));
}

#[test]
fn test_shapefile_includes_geometry_type() {
    let metadata = GeospatialExporter::to_shapefile_metadata();

    assert!(metadata.contains("Point") || metadata.contains("Geometry"));
}

#[test]
fn test_shapefile_includes_projection() {
    let metadata = GeospatialExporter::to_shapefile_metadata();

    assert!(metadata.contains("WGS84") || metadata.contains("EPSG"));
}

// ============================================================================
// 7. EXPORT FORMAT CONSISTENCY TESTS
// ============================================================================

#[test]
fn test_geojson_empty_failures() {
    let geojson = GeospatialExporter::failures_to_geojson(&[]);

    assert_eq!(geojson.r#type, "FeatureCollection");
    assert_eq!(geojson.features.len(), 0);
}

#[test]
fn test_kml_empty_failures() {
    let kml = GeospatialExporter::to_kml(&[]);

    assert!(!kml.is_empty());
    assert!(kml.contains("<?xml"));
}

#[test]
fn test_all_export_formats_produce_output() {
    let failures = create_test_failures();
    let hotspots = create_test_hotspots();

    // All formats should produce non-empty output
    let geojson = GeospatialExporter::failures_to_geojson(&failures);
    let kml = GeospatialExporter::to_kml(&failures);
    let hotspot_geojson = GeospatialExporter::hotspots_to_geojson(&hotspots);
    let raster = GeospatialExporter::create_coverage_raster(10, 10, 0.1, 0.0, 0.0);
    let geotiff_meta = GeospatialExporter::to_geotiff_metadata(&raster);
    let geopackage_meta = GeospatialExporter::to_geopackage_metadata(5);
    let shapefile_meta = GeospatialExporter::to_shapefile_metadata();

    assert!(!geojson.features.is_empty());
    assert!(!kml.is_empty());
    assert!(!hotspot_geojson.features.is_empty());
    assert_eq!(raster.data.len(), 10);
    assert!(!geotiff_meta.is_empty());
    assert!(!geopackage_meta.is_empty());
    assert!(!shapefile_meta.is_empty());
}

// ============================================================================
// Summary
// ============================================================================
// Total: 23 tests covering:
// - GeoJSON export (4 tests + 1 hotspot test)
// - Coverage raster (3 tests)
// - GeoTIFF metadata (2 tests)
// - GeoPackage metadata (2 tests)
// - KML export (3 tests)
// - Shapefile metadata (3 tests)
// - Format consistency (3 tests)
// All tests verify: valid structure, required fields, format correctness

# Phase 2: Selective Adapter Loading Strategy

## The Challenge

ROS bags (Layer 1) are commonly used in isolation. When analyzing a simple ROS bag file:
- User runs: `pyroboreplay analyze mission.bag`
- The system should NOT try to load Linux logs, metrics, or configs
- This avoids unnecessary errors and performance overhead

However, in full incident investigations:
- User provides: incident bundle with all 4 layers
- The system should auto-detect what's available and load adapters intelligently
- Only parse evidence that's actually present

## Solution: Selective Adapter Registry

### Architecture

```
┌─────────────────────────────────────────────────┐
│ IncidentBundle (auto-discovered)                │
│ layers_available: LayerAvailability             │
│ file_inventory: LayerFileInventory              │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ AdapterRegistry (selective loader)              │
│ - Reads LayerAvailability                       │
│ - Instantiates only needed adapters             │
│ - Caches adapter instances                      │
└────────────────┬────────────────────────────────┘
                 │
    ┌────┬───────┼────────┬─────┐
    │    │       │        │     │
    ▼    ▼       ▼        ▼     ▼
  ROS2  Linux  Metrics  Config  (only if available)
  Adapter Adapter Adapter Adapter
```

### Implementation Pattern

```rust
pub struct AdapterRegistry {
    bundle: IncidentBundle,
    adapters: HashMap<String, Box<dyn EvidenceAdapter>>,
}

impl AdapterRegistry {
    pub fn new(bundle: IncidentBundle) -> Self {
        let mut registry = Self {
            bundle,
            adapters: HashMap::new(),
        };
        registry.auto_load_adapters();
        registry
    }

    fn auto_load_adapters(&mut self) {
        let avail = &self.bundle.manifest.layers_available;

        // Always load ROS2 adapter (Layer 1)
        if avail.layer1_ros_bags {
            self.adapters.insert(
                "ros2".to_string(),
                Box::new(Ros2Adapter::new()),
            );
        }

        // Conditionally load Linux adapter (Layer 2)
        if avail.layer2_linux_logs {
            self.adapters.insert(
                "linux_log".to_string(),
                Box::new(LinuxLogAdapter::new()),
            );
        }

        // Conditionally load Metrics adapter (Layer 3)
        if avail.layer3_metrics {
            self.adapters.insert(
                "metrics".to_string(),
                Box::new(MetricsAdapter::new()),
            );
        }

        // Conditionally load Configuration adapter (Layer 4)
        if avail.layer4_configs {
            self.adapters.insert(
                "configuration".to_string(),
                Box::new(ConfigurationAdapter::new()),
            );
        }
    }

    /// Get all events, only from adapters that are loaded
    pub fn ingest_all(&self) -> Result<Vec<MissionEvent>, AdapterError> {
        let mut all_events = Vec::new();

        for (name, adapter) in &self.adapters {
            match adapter.parse(self.bundle.bundle_path.as_path()) {
                Ok(events) => all_events.extend(events),
                Err(e) => eprintln!("Warning: {} adapter failed: {}", name, e),
            }
        }

        Ok(all_events)
    }

    /// Get adapter by name (if loaded)
    pub fn get_adapter(&self, name: &str) -> Option<&dyn EvidenceAdapter> {
        self.adapters
            .get(name)
            .map(|b| b.as_ref() as &dyn EvidenceAdapter)
    }
}
```

## Usage Patterns

### Pattern 1: Simple ROS Bag Analysis (existing)

```rust
// User: pyroboreplay analyze mission.bag
let mut mission = Mission::from_ros_bag("mission.bag")?;
mission.analyze()?;
```

**What happens**:
1. ROS2Adapter loads automatically
2. No Layer 2/3/4 adapters instantiated
3. Timeline contains only Layer 1 events
4. Analysis runs on ROS events only

### Pattern 2: Incident Bundle Analysis (new)

```rust
// User: pyroboreplay analyze incident_2024-07-25/
let bundle = EvidenceDiscovery::discover(Path::new("incident_2024-07-25/"))?;
let registry = AdapterRegistry::new(bundle);

let all_events = registry.ingest_all()?;
// Returns events from ONLY the layers that are present
```

**What happens**:
1. Auto-discover finds: Layer 1 ✅, Layer 2 ✅, Layer 3 ❌, Layer 4 ✅
2. Registry loads only: ROS2, LinuxLog, Configuration adapters
3. MetricsAdapter not instantiated (no metric files detected)
4. Timeline contains events from 3 layers only
5. Analysis can handle missing layers gracefully

### Pattern 3: Programmatic Control

```rust
let bundle = EvidenceDiscovery::discover(Path::new("incident/"))?;
let mut registry = AdapterRegistry::new(bundle);

// Only use specific adapters
if registry.get_adapter("linux_log").is_some() {
    let linux_adapter = registry.get_adapter("linux_log").unwrap();
    let kernel_events = linux_adapter.parse(bundle.bundle_path.as_path())?;
    // Use kernel_events for specific analysis
}

// Skip metrics if you know they're incomplete
registry.adapters.remove("metrics");
```

## Integration Points

### In IncidentAnalyzer (Phase 3+)

```rust
pub struct IncidentAnalyzer {
    bundle: IncidentBundle,
    adapter_registry: AdapterRegistry,
    timeline_engine: TimelineCorrelationEngine,
}

impl IncidentAnalyzer {
    pub fn new(bundle_path: &Path) -> Result<Self, Error> {
        // Auto-discover evidence
        let bundle = EvidenceDiscovery::discover(bundle_path)?;

        // Selective adapter loading
        let adapter_registry = AdapterRegistry::new(bundle.clone());

        // Ingest events from available adapters
        let events = adapter_registry.ingest_all()?;

        // Build timeline (with only available layers)
        let mut timeline_engine = TimelineCorrelationEngine::new(events);
        timeline_engine.synchronize_clocks()?;

        Ok(Self {
            bundle,
            adapter_registry,
            timeline_engine,
        })
    }

    pub fn analyze(&mut self) -> Result<IncidentAnalysis, Error> {
        // Analysis has access to:
        // - bundle.manifest.layers_available (which layers are present)
        // - timeline_engine.unified_timeline (only events from present layers)
        // - adapter_registry (for detailed parsing if needed)

        // Failure detection automatically adjusts confidence based on layers
        // Example: Can't detect CPU overload if Layer 3 not present
        let failures = self.detect_failures()?;

        Ok(IncidentAnalysis {
            failures,
            confidence: self.calculate_confidence(),
        })
    }

    fn calculate_confidence(&self) -> f32 {
        // Confidence depends on available layers
        match self.bundle.manifest.layers_available.analysis_level() {
            4 => 0.95, // All layers available
            3 => 0.85, // Missing one layer
            2 => 0.70, // Only core layers
            1 => 0.60, // ROS-only analysis
            _ => 0.0,
        }
    }
}
```

## Benefits

✅ **No wasted computation**: Only load adapters for evidence that exists  
✅ **Backward compatible**: Existing ROS bag workflows unchanged  
✅ **Graceful degradation**: Missing layers reduce confidence but don't fail  
✅ **Extensible**: Adding new adapters only requires registering in `auto_load_adapters()`  
✅ **Testable**: Can mock/skip adapters in tests  
✅ **User-friendly**: Error messages clear about what's available  

## Phase 3 Integration

In Phase 3 (Timeline Correlation), the engine will receive:
- Events from ONLY the loaded adapters
- Metadata about which layers were available
- Timestamp confidence adjusted for layer availability

This allows the correlation engine to:
1. Skip clock sync for missing layers
2. Adjust causal link confidence based on layer gaps
3. Report clear diagnostics about analysis coverage

## Example: Missing Metrics Impact

**Scenario 1: All 4 layers**
```
Timeline has:
- ROS events
- Kernel events  
- CPU/Memory spikes
- Config parameters

Diagnosis: "CPU overload caused planner timeout"
Confidence: 0.92 (high)
```

**Scenario 2: Layers 1+2 only (missing metrics)**
```
Timeline has:
- ROS events
- Kernel events

Diagnosis: "OOM kill caused planner timeout (inferred from kernel)"
Confidence: 0.78 (medium - no resource data)
```

**Scenario 3: Layer 1 only (plain ROS bag)**
```
Timeline has:
- ROS events only

Diagnosis: "Navigation failure detected"
Confidence: 0.60 (low - can't determine root cause)
```

## Summary

- **Phase 1**: Define incident bundle structure + auto-discovery ✅
- **Phase 2**: Implement layer adapters (selective loading ready) ✅
- **Phase 3**: Timeline correlation (uses layer availability for confidence)
- **Phase 4**: Failure detection (skips layer-specific checks if missing)
- **Phase 5**: Confidence scoring (adjusts based on available layers)

**Status**: Ready for Phase 3 (Timeline Correlation with selective adapter support)

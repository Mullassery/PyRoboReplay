pub mod channel;
pub mod fleet_monitor;
pub mod kafka_stub;
pub mod live_diagnostics;
pub mod processor;
pub mod sla;

pub use channel::{create_stream, EventStream, EventStreamConsumer, StreamConfig, StreamEvent};
pub use fleet_monitor::{
    FleetDashboard, FleetDashboardWindow, FleetHealthSummary, FleetMonitor, FleetMonitorConfig,
    HealthTrend, RobotStatus, RobotStatusType,
};
pub use kafka_stub::{KafkaConnector, KafkaStub};
pub use live_diagnostics::{AlertSeverity, DiagnosticsConfig, LiveAlert, LiveDiagnostics};
pub use processor::{AggregationResult, ProcessorConfig, StreamProcessor};
pub use sla::{
    SlaContract, SlaEnforcementReport, SlaMonitor, SlaViolation, SlaViolationSeverity,
    SlaViolationType,
};

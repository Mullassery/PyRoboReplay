pub mod channel;
pub mod processor;
pub mod live_diagnostics;
pub mod kafka_stub;

pub use channel::{StreamEvent, EventStream, EventStreamConsumer, StreamConfig, create_stream};
pub use processor::{StreamProcessor, ProcessorConfig, AggregationResult};
pub use live_diagnostics::{LiveDiagnostics, DiagnosticsConfig, LiveAlert, AlertSeverity};
pub use kafka_stub::{KafkaConnector, KafkaStub};

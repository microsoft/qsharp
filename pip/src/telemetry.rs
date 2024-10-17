// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod events;
pub(crate) mod mock;

use std::sync::OnceLock;

pub(crate) use events::TelemetryEvent::*;
use mock::MockLoggingProvider;
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider};
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions as semcov;

// store telemetry client in a once cell
static TELEMETRY_CLIENT: OnceLock<TelemetryClient> = OnceLock::new();

trait PythonLoggingProvider: Send + Sync {
    fn log(&self, event: events::TelemetryEvent);
    fn flush(&self) {}
}

impl PythonLoggingProvider for opentelemetry_sdk::logs::LoggerProvider {
    fn log(&self, event: events::TelemetryEvent) {
        let logger = self.logger("qsharp.python");
        let event_name = match event {
            CreateStateVectorSimulator => "CreateStateVectorSimulator",
            RunQasm3 => "RunQasm3",
            ResourceEstimateQasm3 => "ResourceEstimateQasm3",
            CompileQasm3ToQir => "CompileQasm3ToQir",
            CompileQasm => "CompileQasm",
            CompileQasm3ToQsharp => "CompileQasm3ToQsharp",
            InitInterpreter => "InitInterpreter",
            SynthesizeCircuit => "SynthesizeCircuit",
        };
        let mut record = logger.create_log_record();
        record.set_severity_number(opentelemetry::logs::Severity::Info);
        record.set_event_name(event_name);
        logger.emit(record);
    }

    fn flush(&self) {
        if let Err(e) = self.shutdown() {
            eprintln!("Failed to flush telemetry: {e:?}");
        }
        opentelemetry::global::shutdown_tracer_provider();
    }
}

struct TelemetryDisabled;

impl PythonLoggingProvider for TelemetryDisabled {
    fn log(&self, _event: events::TelemetryEvent) {
        // do nothing
    }
}

pub struct TelemetryClient {
    /// `None` if telemetry is disabled or unable to initialize
    logger_provider: Box<dyn PythonLoggingProvider>,
}

impl TelemetryClient {
    fn new(test_mode: bool) -> Self {
        if test_mode {
            return Self::from_logger_provider(MockLoggingProvider::new().1);
        }
        if !events::telemetry_enabled() {
            return Self::disable_telemetry();
        }
        let connection_string = "TODO: application insights key";
        let Ok(exporter) = opentelemetry_application_insights::Exporter::new_from_connection_string(
            connection_string,
            reqwest::Client::new(),
        ) else {
            // silently fail if telemetry fails to initialize, since we don't want to crash the
            // application in the case of telemetry failure (no network connection, etc.)
            return Self::disable_telemetry();
        };

        let logger_provider = opentelemetry_sdk::logs::LoggerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_resource(Resource::new(vec![
                KeyValue::new(semcov::resource::SERVICE_NAMESPACE, "qsharp"),
                KeyValue::new(semcov::resource::SERVICE_NAME, "python"),
            ]))
            .build();

        Self::from_logger_provider(logger_provider)
    }

    fn from_logger_provider(logger_provider: impl PythonLoggingProvider + 'static) -> Self {
        Self {
            logger_provider: Box::new(logger_provider),
        }
    }

    pub fn init_mock_logging() {
        let (receiver, provider) = MockLoggingProvider::new();
        assert!(
            TELEMETRY_CLIENT.get().is_none(),
            "Attempted to init mock logging when client already initialized"
        );
        let _ = TELEMETRY_CLIENT.get_or_init(|| TelemetryClient::from_logger_provider(provider));

        *mock::MOCK_LOG_RECEIVER
            .lock()
            .expect("failed to get lock on mock logging receiver") = Some(receiver);
    }

    pub fn send_event(event: events::TelemetryEvent) {
        let client = TELEMETRY_CLIENT.get_or_init(|| TelemetryClient::new(false));
        client.logger_provider.log(event);
    }

    fn disable_telemetry() -> Self {
        Self {
            logger_provider: Box::new(TelemetryDisabled),
        }
    }
}

impl Drop for TelemetryClient {
    fn drop(&mut self) {
        self.logger_provider.flush();
    }
}

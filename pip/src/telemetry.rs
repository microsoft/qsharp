// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod events {
    // Copyright (c) Microsoft Corporation.
    // Licensed under the MIT License.
    //! This module contains definitions for usage telemetry collected by Microsoft.
    //! Data collected is for product improvement and usage understanding only.
    //! To opt-out of telemetry collection, set the environment variable `NO_TELEMETRY=1`.

    pub(super) fn telemetry_enabled() -> bool {
        std::env::var("NO_TELEMETRY").is_err()
    }

    pub enum TelemetryEvent {
        CreateStateVectorSimulator,
    }
}
pub(crate) use events::*;

pub struct TelemetryClient {
    /// `None` if telemetry is disabled or unable to initialize
    client: Option<SdkMeterProvider>,
}

impl TelemetryClient {
    #[cfg(feature = "test-telemetry")]
    pub fn new() -> Self {
        let provider = InMemoryMetricsExporter::new();
        Self { client: Some(provider.into())
    }

    #[cfg(not(feature = "test-telemetry"))]
    pub fn new() -> Self {
        let connection_string = std::env::var("APPLICATIONINSIGHTS_CONNECTION_STRING").unwrap();
        let Ok(exporter) = opentelemetry_application_insights::Exporter::new_from_connection_string(
            &connection_string,
            reqwest::Client::new(),
        ) else {
            // silently fail if telemetry fails to initialize, since we don't want to crash the
            // application in the case of telemetry failure (no network connection, etc.)
            return Self { client: None };
        };

        let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(
            exporter,
            opentelemetry_sdk::runtime::Tokio,
        )
        .build();

        let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_reader(reader)
            // An opentelemetry resource represents the entity that is generating telemetry.
            // In a deployment context, this would be a specific server or something. But in this
            // case, we just mark it as "client" side telemetry, as we aren't collecting any
            // potentially identifying information.
            .with_resource(opentelemetry_sdk::Resource::new([
                opentelemetry::KeyValue::new("qsharp.python", "client"),
            ]))
            .build();

        opentelemetry::global::set_meter_provider(provider.clone());

        Self { client: Some(provider) }
    }

    pub fn send_event(&self, event: events::TelemetryEvent) {
        if self.client.is_none() {
            return;
        }
        match event {
            events::TelemetryEvent::CreateStateVectorSimulator => {
                todo!("logged CreateStateVectorSimulator event");
            }
        }
    }
}

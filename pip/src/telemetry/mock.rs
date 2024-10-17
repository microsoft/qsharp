// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
//! Used for testing telemetry logging.
use std::sync::{Arc, RwLock};

use pyo3::pyfunction;

use super::{events::TelemetryEvent, PythonLoggingProvider, TelemetryClient};

lazy_static::lazy_static! {
    pub(super) static ref MOCK_LOG_RECEIVER: Arc<RwLock<Vec<TelemetryEvent>>> =
    Arc::new(RwLock::new(vec![]));
}

pub(super) struct MockLoggingProvider;

impl MockLoggingProvider {
    pub(super) fn new() -> Self {
        Self
    }
}

impl PythonLoggingProvider for MockLoggingProvider {
    fn log(&self, event: TelemetryEvent) {
        let mut receiver = MOCK_LOG_RECEIVER
            .write()
            .expect("failed to get lock on mock logging receiver");
        receiver.push(event);
    }

    fn mode(&self) -> &'static str {
        "mock"
    }
}

#[pyfunction]
pub fn drain_logs_from_mock() -> String {
    let mut receiver = MOCK_LOG_RECEIVER
        .write()
        .expect("failed to get lock on mock logging receiver");

    receiver
        .drain(..)
        .map(|x| format!("{x:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[pyfunction]
pub fn init_mock_logging() {
    TelemetryClient::init_mock_logging();
}

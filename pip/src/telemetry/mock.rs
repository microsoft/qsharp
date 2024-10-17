// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
//! Used for testing telemetry logging.
use pyo3::pyfunction;

use super::{events::TelemetryEvent, PythonLoggingProvider, TelemetryClient};

pub(super) static MOCK_LOG_RECEIVER: std::sync::Mutex<
    Option<std::sync::mpsc::Receiver<TelemetryEvent>>,
> = std::sync::Mutex::new(None);

pub(super) struct MockLoggingProvider {
    sender: std::sync::mpsc::Sender<TelemetryEvent>,
}

impl MockLoggingProvider {
    pub(super) fn new() -> (std::sync::mpsc::Receiver<TelemetryEvent>, Self) {
        let (sender, receiver) = std::sync::mpsc::channel();
        (receiver, Self { sender })
    }
}
impl PythonLoggingProvider for MockLoggingProvider {
    fn log(&self, event: TelemetryEvent) {
        self.sender.send(event).expect("mock logger failed");
    }
}

#[pyfunction]
pub fn drain_logs_from_mock() -> String {
    let receiver = MOCK_LOG_RECEIVER
        .lock()
        .expect("failed to get lock on mock logging receiver")
        .take()
        .expect("drain_logs_from_mock called before mock logging initialized");
    receiver
        .try_iter()
        .map(|x| format!("{x:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[pyfunction]
pub fn init_mock_logging() {
    TelemetryClient::init_mock_logging();
}

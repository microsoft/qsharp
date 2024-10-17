// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
//! This module contains definitions for usage telemetry collected by Microsoft.
//! Data collected is for product improvement and usage understanding only.
//! To opt-out of telemetry collection, set the environment variable `NO_TELEMETRY=1`.

pub(super) fn telemetry_enabled() -> bool {
    std::env::var("NO_TELEMETRY").is_err()
}

#[derive(Clone, Copy, Debug)]
pub enum TelemetryEvent {
    CreateStateVectorSimulator,
    RunQasm3,
    ResourceEstimateQasm3,
    CompileQasm3ToQir,
    CompileQasm,
    CompileQasm3ToQsharp,
    InitInterpreter,
    SynthesizeCircuit,
}

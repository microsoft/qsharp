// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// See https://serde.rs/enum-representations.html
// Over wasm, this will result in a JavaScript object similar to
// { "id": "Parsed", "data": { "errors": 0, "size": 512 }}
#[derive(Serialize, Deserialize)]
#[serde(tag = "id", content = "data")]
pub enum Event<'a> {
    Loaded { git_hash: &'a str },
    Panic { details: &'a str },
    Parsed { errors: u32, size: u64 },
    Run { shots: u32, size: u64 },
    Eval { qubits: u64, expression_count: u64 },
}

pub trait Log: Sync + Send {
    fn log(&self, event: &Event);
}

// Use the Atomic bool for low-overhead checking if telemetry is enabled before unwrapping the logger
static TELEM_ENABLED: AtomicBool = AtomicBool::new(false);
static TELEM_GLOBAL: OnceLock<&dyn Log> = OnceLock::new();

/// # Errors
///
/// Will return an error if the telemetry logger has already been set
pub fn set_telemetry_logger(logger: &'static dyn Log) -> Result<(), &str> {
    TELEM_GLOBAL
        .set(logger)
        .map_err(|_| "attempted to set a telemetry logger after it was already assigned")?;
    TELEM_ENABLED.store(true, Ordering::Release);
    Ok(())
}

#[inline]
pub fn is_telemetry_enabled() -> bool {
    TELEM_ENABLED.load(Ordering::Acquire)
}

pub fn log(event: &Event) {
    if is_telemetry_enabled() {
        if let Some(logger) = TELEM_GLOBAL.get() {
            logger.log(event);
        }
    }
}

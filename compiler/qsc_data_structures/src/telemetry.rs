use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;

pub trait Log: Sync + Send {
    fn log(&self, msg: &str);
}

// Use the Atomic bool for low-overhead checking if telemetry is enabled before unwrapping the logger
static TELEM_ENABLED: AtomicBool = AtomicBool::new(false);
static TELEM_GLOBAL: OnceLock<&dyn Log> = OnceLock::new();

pub fn set_telemetry_logger(logger: &'static dyn Log) {
    let _ = TELEM_GLOBAL.set(logger); // TODO: Error handling
    TELEM_ENABLED.store(true, Ordering::Release);
}

#[inline]
pub fn is_telemetry_enabled() -> bool {
    TELEM_ENABLED.load(Ordering::Acquire)
}

pub fn log(msg: &str) {
    if is_telemetry_enabled() {
        if let Some(logger) = TELEM_GLOBAL.get() {
            logger.log(msg);
        }
    }
}

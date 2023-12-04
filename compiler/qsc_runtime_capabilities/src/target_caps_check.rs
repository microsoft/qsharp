use crate::StoreCapabilities;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_fir::fir::PackageStore;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("target does not support capability {0}")]
    #[diagnostic(code("Qsc.RuntimeCapabilities.UnsupportedCapability"))]
    UnsupportedCapability(String, #[label] Span),
}

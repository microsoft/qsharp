use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("found a qsharp.json file, but it was invalid: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to walk the project directory to discover Q# source files: {0}")]
    FsWalk(#[from] globwalk::GlobError),
    #[error("failed to construct regular expression from excluded file item: {0}")]
    RegexError(#[from] regex_lite::Error),
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    FsWalk(#[from] globwalk::GlobError),
}

impl miette::Diagnostic for Error {}

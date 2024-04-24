// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use miette::IntoDiagnostic;

pub trait SourceResolver {
    #[cfg(feature = "fs")]
    fn resolve<P>(&self, path: P) -> miette::Result<(PathBuf, String)>
    where
        P: AsRef<Path>,
    {
        let path = std::fs::canonicalize(path).map_err(|e| {
            crate::Error(crate::ErrorKind::IO(format!(
                "Could not resolve include file path: {e}"
            )))
        })?;
        match std::fs::read_to_string(&path) {
            Ok(source) => Ok((path, source)),
            Err(_) => Err(crate::Error(crate::ErrorKind::NotFound(format!(
                "Could not resolve include file: {}",
                path.display()
            ))))
            .into_diagnostic(),
        }
    }
    #[cfg(not(feature = "fs"))]
    fn resolve<P>(&self, path: P) -> miette::Result<(PathBuf, String)>
    where
        P: AsRef<Path>;
}

#[cfg(feature = "fs")]
pub mod fs {
    use super::SourceResolver;

    #[derive(Default)]
    pub struct FsSourceResolver;
    impl SourceResolver for FsSourceResolver {}
}

pub struct InMemorySourceResolver {
    sources: FxHashMap<PathBuf, String>,
}

impl FromIterator<(Arc<str>, Arc<str>)> for InMemorySourceResolver {
    fn from_iter<T: IntoIterator<Item = (Arc<str>, Arc<str>)>>(iter: T) -> Self {
        let mut map = FxHashMap::default();
        for (path, source) in iter {
            map.insert(PathBuf::from(path.to_string()), source.to_string());
        }

        InMemorySourceResolver { sources: map }
    }
}

impl SourceResolver for InMemorySourceResolver {
    fn resolve<P>(&self, path: P) -> miette::Result<(PathBuf, String)>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        match self.sources.get(path) {
            Some(source) => Ok((path.to_owned(), source.clone())),
            None => Err(crate::Error(crate::ErrorKind::NotFound(format!(
                "Could not resolve include file: {}",
                path.display()
            ))))
            .into_diagnostic(),
        }
    }
}

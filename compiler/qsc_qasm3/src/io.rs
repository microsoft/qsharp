// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use miette::IntoDiagnostic;

/// A trait for resolving include file paths to their contents.
/// This is used by the parser to resolve `include` directives.
/// Implementations of this trait can be provided to the parser
/// to customize how include files are resolved.
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

/// A source resolver that resolves include files from an in-memory map.
/// This is useful for testing or environments in which file system access
/// is not available.
///
/// This requires users to build up a map of include file paths to their
/// contents prior to parsing.
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

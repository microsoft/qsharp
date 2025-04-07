// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod error;
pub use error::Error;
pub use error::ErrorKind;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

/// A trait for resolving include file paths to their contents.
/// This is used by the parser to resolve `include` directives.
/// Implementations of this trait can be provided to the parser
/// to customize how include files are resolved.
pub trait SourceResolver {
    fn ctx(&mut self) -> &mut SourceResolverContext;

    #[cfg(feature = "fs")]
    fn resolve<P>(&mut self, path: P) -> miette::Result<(PathBuf, String), Error>
    where
        P: AsRef<Path>,
    {
        let path = std::fs::canonicalize(path).map_err(|e| {
            Error(ErrorKind::IO(format!(
                "Could not resolve include file path: {e}"
            )))
        })?;

        // If the new path makes a cycle in the include graph, we return
        // an error showing the cycle to the user.
        if let Some(cycle) = self.ctx().cycle_made_by_including_path(&path) {
            return Err(Error(ErrorKind::CyclicInclude(cycle)));
        }

        // If the new path doesn't make a cycle but it was already
        // included before, we return a `MultipleInclude`
        // error saying "<FILE> was already included in <FILE>".
        if let Some(parent_file) = self.ctx().path_was_already_included(&path) {
            return Err(Error(ErrorKind::MultipleInclude(
                parent_file.display().to_string(),
            )));
        }

        match std::fs::read_to_string(&path) {
            Ok(source) => {
                self.ctx().add_path_to_include_graph(&path);
                Ok((path, source))
            }
            Err(_) => Err(Error(ErrorKind::NotFound(format!(
                "Could not resolve include file: {}",
                path.display()
            )))),
        }
    }
    #[cfg(not(feature = "fs"))]
    fn resolve<P>(&self, path: P) -> miette::Result<(PathBuf, String)>
    where
        P: AsRef<Path>;
}

#[derive(Default)]
pub struct SourceResolverContext {
    // A graph representation of the include chain.
    include_graph: FxHashMap<PathBuf, Vec<PathBuf>>,
    current_path: Option<PathBuf>,
}

impl SourceResolverContext {
    /// If including the path makes a cycle, returns a vector of the paths that make the cycle.
    /// Else, returns None.
    ///
    /// This implementation uses Tarjan's strongly connected components algorithm.
    /// Reference: <https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm>.
    fn cycle_made_by_including_path(&mut self, path: &PathBuf) -> Option<Cycle> {
        todo!()
    }

    /// If the path was already included, returns the path of the file that included it.
    /// Else, returns None.
    fn path_was_already_included(&mut self, path: &PathBuf) -> Option<PathBuf> {
        todo!()
    }

    /// Adds `path` as a child of `current_path`, and then changes the `current_path` to `path`.
    fn add_path_to_include_graph(&mut self, path: &PathBuf) {
        todo!()
    }

    /// Changes `current_path` to its parent in the `include_graph`.
    pub fn pop_current_file(&mut self) {
        todo!()
    }
}

/// We use this struct to print a nice error message when we find a cycle.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cycle {
    paths: Vec<PathBuf>,
}

impl std::fmt::Display for Cycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// A source resolver that resolves include files from an in-memory map.
/// This is useful for testing or environments in which file system access
/// is not available.
///
/// This requires users to build up a map of include file paths to their
/// contents prior to parsing.
pub struct InMemorySourceResolver {
    sources: FxHashMap<PathBuf, String>,
    ctx: SourceResolverContext,
}

impl FromIterator<(Arc<str>, Arc<str>)> for InMemorySourceResolver {
    fn from_iter<T: IntoIterator<Item = (Arc<str>, Arc<str>)>>(iter: T) -> Self {
        let mut map = FxHashMap::default();
        for (path, source) in iter {
            map.insert(PathBuf::from(path.to_string()), source.to_string());
        }

        InMemorySourceResolver {
            sources: map,
            ctx: Default::default(),
        }
    }
}

impl SourceResolver for InMemorySourceResolver {
    fn ctx(&mut self) -> &mut SourceResolverContext {
        &mut self.ctx
    }

    fn resolve<P>(&mut self, path: P) -> miette::Result<(PathBuf, String), Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // If the new path makes a cycle in the include graph, we return
        // an error showing the cycle to the user.
        if let Some(cycle) = self.ctx.cycle_made_by_including_path(&path_buf) {
            return Err(Error(ErrorKind::CyclicInclude(cycle)));
        }

        // If the new path doesn't make a cycle but it was already
        // included before, we return a `MultipleInclude`
        // error saying "<FILE> was already included in <FILE>".
        if let Some(parent_file) = self.ctx.path_was_already_included(&path_buf) {
            return Err(Error(ErrorKind::MultipleInclude(
                parent_file.display().to_string(),
            )));
        }

        match self.sources.get(path) {
            Some(source) => {
                self.ctx.add_path_to_include_graph(&path_buf);
                Ok((path.to_owned(), source.clone()))
            }
            None => Err(Error(ErrorKind::NotFound(format!(
                "Could not resolve include file: {}",
                path.display()
            )))),
        }
    }
}

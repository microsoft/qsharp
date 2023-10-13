mod source;

use source::Source;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ResolverIoError(#[from] std::io::Error),
}

/// Given a single Q# source, returns all discovered sources that are a part of that compilation unit.
/// Does not return the input source as an additional source.
/// Using either a given path, or if none is specified, the current working directory:
/// 1. find the corresponding manifest file
/// 2a. if there is a manifest file, include <manifest_dir>/**/*.qs in the sources
/// 2b. if there is no manifest file, return an empty list, denoting single-file compilation mode.
pub fn find_dependencies_with_loader<FileLoader>(
    path: Option<PathBuf>,
    load_module: FileLoader,
) -> miette::Result<Vec<(Arc<str>, Arc<str>)>>
where
    for<'a> FileLoader: Fn(&'a PathBuf) -> miette::Result<(Arc<str>, Arc<str>)>,
{
    let manifest = match qsc_manifest::find_manifest()? {
        Some(manifest) => manifest,
        None => return Ok(Default::default()),
    };

    let glob = globwalk::GlobWalkerBuilder::from_patterns(manifest.manifest_dir, &["**/*.qs"])
        .follow_links(false)
        .build()?
        .into_iter()
        .filter_map(Result::ok);

    for file in glob {
        todo!();
    }

    todo!("**/*.qs in manfiest dir");
}

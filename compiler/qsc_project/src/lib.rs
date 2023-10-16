mod fs;
mod manifest;

pub use fs::find_manifest;
pub use manifest::{Manifest, ManifestDescriptor, MANIFEST_FILE_NAME};

use std::{path::PathBuf, sync::Arc};

/// Given a single Q# source, returns all discovered sources that are a part of that compilation unit.
/// Does not return the input source as an additional source.
/// Using either a given path, or if none is specified, the current working directory:
/// 1. find the corresponding manifest file
/// 2a. if there is a manifest file, include <manifest_dir>/**/*.qs in the sources
/// 2b. if there is no manifest file, return an empty list, denoting single-file compilation mode.
pub fn find_dependencies_with_loader<FileLoader>(
    load_module: FileLoader,
) -> miette::Result<Vec<(Arc<str>, Arc<str>)>>
where
    for<'a> FileLoader: Fn(&'a PathBuf) -> miette::Result<(Arc<str>, Arc<str>)>,
{
    let manifest = match find_manifest()? {
        Some(manifest) => manifest,
        None => return Ok(Default::default()),
    };

    let qs_files = globwalk::GlobWalkerBuilder::from_patterns(manifest.manifest_dir, &["*.qs"])
        .build()
        .map_err(Into::<crate::fs::Error>::into)?
        .filter_map(Result::ok)
        .filter(|item| {
            !manifest
                .manifest
                .exclude_files
                .iter()
                .any(|x| Some(x.as_str()) == item.file_name().to_str())
        });

    let qs_files = qs_files.into_iter().map(|file| file.path().into());

    let qs_sources = qs_files.map(|path| load_module(&path));

    qs_sources.collect::<miette::Result<_>>()
}

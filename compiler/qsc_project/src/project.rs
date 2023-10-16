use std::{path::PathBuf, sync::Arc};

use crate::Manifest;

/// Describes a Q# project
#[derive(Default, Debug)]
pub struct Project {
    pub sources: Vec<(Arc<str>, Arc<str>)>,
    pub manifest: crate::Manifest,
}

impl Project {
    /// Find all members of a Q# project and load them.
    /// 1. find the corresponding manifest file
    /// 2a. if there is a manifest file, include <manifest_dir>/**/*.qs in the sources
    /// 2b. if there is no manifest file, return an empty list, denoting single-file compilation mode.
    /// 3. exclude any explicitly excluded files in the manifest
    pub fn load<FileLoader>(read_file: FileLoader) -> miette::Result<Project>
    where
        for<'a> FileLoader: Fn(&'a PathBuf) -> miette::Result<(Arc<str>, Arc<str>)>,
    {
        let manifest = match Manifest::load()? {
            Some(manifest) => manifest,
            None => return Ok(Default::default()),
        };

        let mut patterns = Vec::with_capacity(manifest.manifest.exclude_files.len() + 1);
        patterns.push("*.qs".to_string());

        let patterns_to_exclude = manifest
            .manifest
            .exclude_files
            .iter()
            .map(|item| format!("!{item}"));

        patterns.extend(patterns_to_exclude);

        let qs_files =
            globwalk::GlobWalkerBuilder::from_patterns(manifest.manifest_dir, &patterns[..])
                .build()
                .map_err(Into::<crate::Error>::into)?
                .filter_map(Result::ok)
                .filter(|item| {
                    !manifest
                        .manifest
                        .exclude_files
                        .iter()
                        .any(|x| Some(x.as_str()) == item.file_name().to_str())
                });

        let qs_files = qs_files.into_iter().map(|file| file.path().into());

        let qs_sources = qs_files.map(|path| read_file(&path));

        let sources = qs_sources.collect::<miette::Result<_>>()?;
        Ok(Project {
            manifest: manifest.manifest,
            sources,
        })
    }

    pub fn load_from_path<FileLoader>(
        path: PathBuf,
        read_file: FileLoader,
    ) -> miette::Result<Project>
    where
        for<'a> FileLoader: Fn(&'a PathBuf) -> miette::Result<(Arc<str>, Arc<str>)>,
    {
        let manifest = match Manifest::load_from_path(path)? {
            Some(manifest) => manifest,
            None => return Ok(Default::default()),
        };

        let mut patterns = Vec::with_capacity(manifest.manifest.exclude_files.len() + 1);
        patterns.push("*.qs".to_string());

        let patterns_to_exclude = manifest
            .manifest
            .exclude_files
            .iter()
            .map(|item| format!("!{item}"));

        patterns.extend(patterns_to_exclude);

        let qs_files =
            globwalk::GlobWalkerBuilder::from_patterns(manifest.manifest_dir, &patterns[..])
                .build()
                .map_err(Into::<crate::Error>::into)?
                .filter_map(Result::ok)
                .filter(|item| {
                    !manifest
                        .manifest
                        .exclude_files
                        .iter()
                        .any(|x| Some(x.as_str()) == item.file_name().to_str())
                });

        let qs_files = qs_files.into_iter().map(|file| file.path().into());

        let qs_sources = qs_files.map(|path| read_file(&path));

        let sources = qs_sources.collect::<miette::Result<_>>()?;
        Ok(Project {
            manifest: manifest.manifest,
            sources,
        })
    }
}

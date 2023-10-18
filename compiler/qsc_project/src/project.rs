use std::{path::PathBuf, sync::Arc};

use regex_lite::Regex;

use crate::Manifest;

/// Describes a Q# project
#[derive(Default, Debug)]
pub struct Project {
    pub sources: Vec<(Arc<str>, Arc<str>)>,
    pub manifest: crate::Manifest,
}

#[derive(PartialEq)]
pub enum EntryType {
    File,
    Folder,
}

pub trait DirEntry {
    fn entry_type(&self) -> EntryType;
    fn extension(&self) -> String;
    fn entry_name(&self) -> String;
    fn path(&self) -> PathBuf;
}

pub trait FileSystem<T: DirEntry> {
    fn read_file(&self, path: &PathBuf) -> miette::Result<(Arc<str>, Arc<str>)>;
    fn list_directory(&self, path: &PathBuf) -> miette::Result<Vec<T>>;

    fn fetch_files_with_exclude_pattern(
        &self,
        exclude_patterns: &[Regex],
        initial_path: &PathBuf,
    ) -> miette::Result<Vec<T>> {
        let listing = self.list_directory(initial_path)?;
        let mut files = vec![];
        for item in listing {
            let name = item.path().to_string_lossy().to_string();
            if regex_matches(exclude_patterns, &name) {
                continue;
            }
            match item.entry_type() {
                EntryType::File if item.extension() == ".qs" => files.push(item),
                EntryType::Folder => files.append(
                    &mut self.fetch_files_with_exclude_pattern(exclude_patterns, &item.path())?,
                ),
                _ => (),
            }
        }
        Ok(files)
    }
    fn load(&self) -> miette::Result<Project> {
        // TODO: pass in manifest
        let manifest = match Manifest::load()? {
            Some(manifest) => manifest,
            None => return Ok(Default::default()),
        };

        let exclude_patterns: Vec<_> = manifest
            .manifest
            .exclude_files
            .iter()
            .map(|x| Regex::new(x))
            .collect::<Result<_, _>>()
            .map_err(crate::Error::from)?;

        let qs_files =
            self.fetch_files_with_exclude_pattern(&exclude_patterns, &manifest.manifest_dir)?;

        let qs_files = qs_files.into_iter().map(|file| file.path().into());

        let qs_sources = qs_files.map(|path| self.read_file(&path));

        let sources = qs_sources.collect::<miette::Result<_>>()?;
        Ok(Project {
            manifest: manifest.manifest,
            sources,
        })
    }

    fn load_from_path<FileLoader>(path: PathBuf, read_file: FileLoader) -> miette::Result<Project>
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

fn regex_matches(exclude_patterns: &[Regex], entry_name: &str) -> bool {
    todo!()
}

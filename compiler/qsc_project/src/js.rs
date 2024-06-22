// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{DirEntry, EntryType, FileSystemAsync};
use async_trait::async_trait;
use miette::Error;
use std::{convert::Infallible, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct JSFileEntry {
    pub name: String,
    pub r#type: EntryType,
}

impl DirEntry for JSFileEntry {
    type Error = Infallible;

    fn entry_type(&self) -> Result<EntryType, Self::Error> {
        Ok(self.r#type)
    }

    fn path(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}

#[async_trait(?Send)]
pub trait ProjectHost {
    async fn read_file_(&self, uri: &str) -> (Arc<str>, Arc<str>);
    async fn list_directory_(&self, dir_uri: &str) -> Vec<JSFileEntry>;
    async fn resolve_path_(&self, base: &str, path: &str) -> Option<Arc<str>>;
    async fn fetch_github_(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> Option<Arc<str>>;
    async fn find_manifest_directory(&self, doc_uri: &str) -> Option<Arc<str>>;
}

#[async_trait(?Send)]
impl<T> FileSystemAsync for T
where
    T: ProjectHost + ?Sized,
{
    type Entry = JSFileEntry;

    async fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        return Ok(self.read_file_(&path.to_string_lossy()).await);
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        return Ok(self.list_directory_(&path.to_string_lossy()).await);
    }

    async fn resolve_path(
        &self,
        base: &std::path::Path,
        path: &std::path::Path,
    ) -> miette::Result<std::path::PathBuf> {
        let res = self
            .resolve_path_(&base.to_string_lossy(), &path.to_string_lossy())
            .await
            .ok_or(Error::msg("Path could not be resolved"))?;
        return Ok(PathBuf::from(res.to_string()));
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<std::sync::Arc<str>> {
        let content = self
            .fetch_github_(owner, repo, r#ref, path)
            .await
            .ok_or(Error::msg("Github content could not be retrieved"))?;
        return Ok(content);
    }
}

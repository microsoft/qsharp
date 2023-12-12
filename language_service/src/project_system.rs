// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::state::CompilationStateUpdater;
use async_trait::async_trait;
use std::{convert::Infallible, path::PathBuf};

#[derive(Debug)]
pub struct JSFileEntry {
    pub name: String,
    pub r#type: qsc_project::EntryType,
}

impl qsc_project::DirEntry for JSFileEntry {
    type Error = Infallible;

    fn entry_type(&self) -> Result<qsc_project::EntryType, Self::Error> {
        Ok(self.r#type)
    }

    fn path(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}

#[async_trait(?Send)]
impl qsc_project::FileSystemAsync for CompilationStateUpdater<'_> {
    type Entry = JSFileEntry;
    async fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        Ok((self.read_file_callback)(path.to_string_lossy().to_string()).await)
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok((self.list_directory)(path.to_string_lossy().to_string()).await)
    }
}

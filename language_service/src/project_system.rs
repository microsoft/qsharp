// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{DirEntry, EntryType};
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
/// the desugared return type of an "async fn"
type PinnedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// represents a unary async fn where `Arg` is the input
/// parameter and `Return` is the return type. The lifetime
/// `'a` represents the lifetime of the contained `dyn Fn`.
type AsyncFunction<'a, Arg, Return> = Box<dyn Fn(Arg) -> PinnedFuture<Return> + 'a>;
use std::{future::Future, pin::Pin};

pub struct ProjectSystemCallbacks<'a> {
    /// Callback which lets the service read a file from the target filesystem
    pub read_file: AsyncFunction<'a, String, (Arc<str>, Arc<str>)>,
    /// Callback which lets the service list directory contents
    /// on the target file system
    pub list_directory: AsyncFunction<'a, String, Vec<JSFileEntry>>,
    /// Fetch the manifest file for a specific path
    pub get_manifest: AsyncFunction<'a, String, Option<crate::ManifestDescriptor>>,
}

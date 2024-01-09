// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::compilation::Compilation;
use super::protocol::{DiagnosticUpdate, NotebookMetadata};
use crate::protocol::WorkspaceConfigurationUpdate;
use log::{error, trace};
use miette::Diagnostic;
use qsc::{compile::Error, target::Profile, PackageType};
use qsc_project::{FileSystemAsync, JSFileEntry};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cell::RefCell, fmt::Debug, future::Future, mem::take, pin::Pin, rc::Rc, sync::Arc};

/// the desugared return type of an "async fn"
type PinnedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// represents a unary async fn where `Arg` is the input
/// parameter and `Return` is the return type. The lifetime
/// `'a` represents the lifetime of the contained `dyn Fn`.
type AsyncFunction<'a, Arg, Return> = Box<dyn Fn(Arg) -> PinnedFuture<Return> + 'a>;

#[derive(Default, Debug)]
pub(super) struct CompilationState {
    /// A `CompilationUri` is an identifier for a unique compilation.
    /// It is NOT required to be a uri that represents an actual document.
    ///
    /// For single Q# documents, the `CompilationUri` is the same as the
    /// document uri.
    ///
    /// For notebooks, the `CompilationUri` is the notebook uri.
    ///
    /// The `CompilatinUri` is used when compilation-level errors get reported
    /// to the client. Compilation-level errors are defined as errors without
    /// an associated source document.
    ///
    /// `PartialConfiguration` contains configuration overrides for this
    /// compilation, explicitly specified through a project manifest (not currently implemented)
    /// or notebook metadata.
    compilations: FxHashMap<CompilationUri, (Compilation, PartialConfiguration)>,
    /// All the documents that we were told about by the client.
    ///
    /// This map doesn't necessarily contain ALL the documents that
    /// make up a compilation - only the ones that are currently open.
    open_documents: FxHashMap<DocumentUri, OpenDocument>,
}

type CompilationUri = Arc<str>;
type DocumentUri = Arc<str>;

#[derive(Debug)]
struct OpenDocument {
    /// This version is the document version provided by the client.
    /// It increases strictly with each text change, though this knowledge should
    /// not be important. The version is only ever used when publishing
    /// diagnostics to help the client associate the list of diagnostics
    /// with a snapshot of the document.
    pub version: u32,
    pub compilation: CompilationUri,
    pub latest_str_content: Arc<str>,
}

#[derive(Debug, Copy, Clone)]
struct Configuration {
    pub target_profile: Profile,
    pub package_type: PackageType,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            target_profile: Profile::Unrestricted,
            package_type: PackageType::Exe,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct PartialConfiguration {
    pub target_profile: Option<Profile>,
    pub package_type: Option<PackageType>,
}

pub(super) struct CompilationStateUpdater<'a> {
    /// Compilation state which is shared with readers. It can only be accessed
    /// by dynamically borrowing. Mutable references to `CompilationState` should not
    /// be held across `await` points since that can cause readers to be denied access.
    state: Rc<RefCell<CompilationState>>,
    /// Workspace-wide configuration settings. These can include compiler settings that
    /// affect error checking and other language server behavior.
    ///
    /// Some settings can be set both at the compilation scope and at the workspace scope.
    /// Compilation-scoped settings take precedence over workspace-scoped settings.
    configuration: Configuration,
    /// Documents that we have previously published errors about. We need to
    /// keep track of this so we can clear errors from them when documents are removed
    /// from a compilation or when a recompilation occurs.
    documents_with_errors: FxHashSet<DocumentUri>,
    /// Callback which will receive diagnostics (compilation errors)
    /// whenever a (re-)compilation occurs.
    diagnostics_receiver: Box<dyn Fn(DiagnosticUpdate) + 'a>,
    /// Callback which lets the service read a file from the target filesystem
    pub(crate) read_file_callback: AsyncFunction<'a, String, (Arc<str>, Arc<str>)>,
    /// Callback which lets the service list directory contents
    /// on the target file system
    pub(crate) list_directory: AsyncFunction<'a, String, Vec<JSFileEntry>>,
    /// Fetch the manifest file for a specific path
    get_manifest: AsyncFunction<'a, String, Option<qsc_project::ManifestDescriptor>>,
}

impl<'a> CompilationStateUpdater<'a> {
    pub fn new(
        state: Rc<RefCell<CompilationState>>,
        diagnostics_receiver: impl Fn(DiagnosticUpdate) + 'a,
        read_file: impl Fn(String) -> Pin<Box<dyn Future<Output = (Arc<str>, Arc<str>)>>> + 'a,
        list_directory: impl Fn(String) -> Pin<Box<dyn Future<Output = Vec<JSFileEntry>>>> + 'a,
        get_manifest: impl Fn(String) -> Pin<Box<dyn Future<Output = Option<qsc_project::ManifestDescriptor>>>>
            + 'a,
    ) -> Self {
        Self {
            state,
            configuration: Configuration::default(),
            documents_with_errors: FxHashSet::default(),
            diagnostics_receiver: Box::new(diagnostics_receiver),
            read_file_callback: Box::new(read_file),
            list_directory: Box::new(list_directory),
            get_manifest: Box::new(get_manifest),
        }
    }

    /// Updates the workspace configuration. If any compiler settings are updated,
    /// a recompilation may be triggered, which will result in a new set of diagnostics
    /// being published.
    pub fn update_configuration(&mut self, configuration: WorkspaceConfigurationUpdate) {
        let need_recompile = self.apply_configuration(configuration);

        // Some configuration options require a recompilation as they impact error checking
        if need_recompile {
            self.recompile_all();
        }
    }

    pub(super) async fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        let doc_uri: Arc<str> = Arc::from(uri);
        let text: Arc<str> = Arc::from(text);

        let project = self.load_manifest(&doc_uri).await;

        let (compilation_uri, sources) = project.unwrap_or_else(|| {
            // If we are in single file mode, use the file's path as the compilation identifier.
            (doc_uri.clone(), vec![(doc_uri.clone(), text.clone())])
        });

        let prev_compilation_uri = self.with_state_mut(|state| {
            state
                .open_documents
                .insert(
                    doc_uri.clone(),
                    OpenDocument {
                        version,
                        compilation: compilation_uri.clone(),
                        latest_str_content: text,
                    },
                )
                .map(|d| d.compilation)
        });

        // If a document switched compilations, we may need to remove the compilation
        // it previously belonged to.
        if let Some(prev_compilation_uri) = prev_compilation_uri {
            if prev_compilation_uri != compilation_uri {
                self.maybe_close_project(&prev_compilation_uri);
            }
        }

        self.insert_buffer_aware_compilation(sources, &compilation_uri);

        self.publish_diagnostics();
    }

    /// Attempts to resolve a manifest for the given document uri.
    /// If a manifest is found, returns the manifest uri along
    /// with the sources for the project
    async fn load_manifest(
        &self,
        doc_uri: &Arc<str>,
    ) -> Option<(Arc<str>, Vec<(Arc<str>, Arc<str>)>)> {
        let manifest = (self.get_manifest)(doc_uri.to_string()).await;
        if let Some(ref manifest) = manifest {
            let res = self.load_project(manifest).await;
            match res {
                Ok(o) => Some((manifest.compilation_uri(), o.sources)),
                Err(e) => {
                    error!("failed to load manifest: {e:?}, defaulting to single-file mode");
                    None
                }
            }
        } else {
            trace!("Running in single file mode");
            None
        }
    }

    /// This function takes a vector of sources and creates a compilation out of them.
    /// It checks currently open documents and uses those buffers instead of any
    /// sources provided in the vector, effectively prioritizing open document contents
    /// over fs contents.
    fn insert_buffer_aware_compilation(
        &mut self,
        mut sources: Vec<(Arc<str>, Arc<str>)>,
        compilation_uri: &Arc<str>,
    ) {
        self.with_state_mut(|state| {
            // replace source with one from memory if it exists
            // this is what prioritizes open buffers over what exists on the fs for a
            // given document
            for (ref l_uri, ref mut source) in &mut sources {
                if let Some(doc) = state.open_documents.get(l_uri) {
                    trace!("{l_uri} is open, using source from open document");
                    *source = doc.latest_str_content.clone();
                }
            }

            let compilation = Compilation::new(
                &sources,
                self.configuration.package_type,
                self.configuration.target_profile,
            );

            state.compilations.insert(
                compilation_uri.clone(),
                (compilation, PartialConfiguration::default()),
            );
        });
    }

    pub(super) async fn close_document(&mut self, uri: &str) {
        let project = self.load_manifest(&uri.into()).await;

        let removed_compilation = self.remove_open_document(uri);

        if !removed_compilation {
            // If the project is still open, update it so that it
            // uses the disk contents instead of the open buffer contents
            // for this document
            if let Some(project) = project {
                self.insert_buffer_aware_compilation(project.1, &project.0);
            }
        }

        self.publish_diagnostics();
    }

    /// Removes a document from the open documents map. If the
    /// document was the last open document in a compilation,
    /// the compilation is also removed.
    fn remove_open_document(&mut self, uri: &str) -> bool {
        let existing_compilation_uri = self.with_state_mut(|state| {
            state.compilations.remove(uri);

            state
                .open_documents
                .remove(uri)
                .expect("document should exist")
                .compilation
        });
        self.maybe_close_project(&existing_compilation_uri)
    }

    fn maybe_close_project(&mut self, compilation_uri: &Arc<str>) -> bool {
        self.with_state_mut(|state| {
            // if there are no remaining open documents with the project's compilation URI
            if state
                .open_documents
                .iter()
                .all(|(_uri, doc)| doc.compilation != *compilation_uri)
            {
                trace!("closing project {:?}", compilation_uri);
                state.compilations.remove(compilation_uri);
                return true;
            }
            false
        })
    }

    pub(super) fn update_notebook_document<'b, I>(
        &mut self,
        notebook_uri: &str,
        notebook_metadata: NotebookMetadata,
        cells: I,
    ) where
        I: Iterator<Item = (&'b str, u32, &'b str)>, // uri, version, text - basically DidChangeTextDocumentParams in LSP
    {
        self.with_state_mut(|state| {
            let compilation_uri: Arc<str> = notebook_uri.into();

            // First remove all previously known cells for this notebook
            state
                .open_documents
                .retain(|_, open_doc| notebook_uri != open_doc.compilation.as_ref());

            let notebook_configuration = PartialConfiguration {
                target_profile: notebook_metadata.target_profile,
                package_type: None,
            };
            let configuration = merge_configurations(notebook_configuration, self.configuration);

            // Compile the notebook and add each cell into the document map
            let compilation = Compilation::new_notebook(
                cells.map(|(cell_uri, version, cell_contents)| {
                    trace!("update_notebook_document: cell: {cell_uri} {version}");
                    state.open_documents.insert(
                        (*cell_uri).into(),
                        OpenDocument {
                            version,
                            compilation: compilation_uri.clone(),
                            latest_str_content: Arc::from(cell_contents),
                        },
                    );
                    (Arc::from(cell_uri), Arc::from(cell_contents))
                }),
                configuration.target_profile,
            );

            state.compilations.insert(
                compilation_uri.clone(),
                (compilation, notebook_configuration),
            );
        });
        self.publish_diagnostics();
    }

    pub(super) fn close_notebook_document(&mut self, notebook_uri: &str) {
        self.with_state_mut(|state| {
            trace!("close_notebook_document: {notebook_uri}");

            // Cells for the notebook are kept in the open documents map.
            // First remove all the cells for the notebook from the open
            // documents map.
            state
                .open_documents
                .retain(|_, open_doc| notebook_uri != open_doc.compilation.as_ref());

            // Then remove the notebook itself from the compilations map
            state.compilations.remove(notebook_uri);
        });

        self.publish_diagnostics();
    }

    // It gets really messy knowing when to clear diagnostics
    // when the document changes ownership between compilations, etc.
    // So let's do it the simplest way possible. Republish all the diagnostics every time.
    fn publish_diagnostics(&mut self) {
        let last_docs_with_errors = take(&mut self.documents_with_errors);
        let mut docs_with_errors = FxHashSet::default();

        self.with_state(|state| {
            for (compilation_uri, compilation) in &state.compilations {
                trace!("publishing diagnostics for {compilation_uri}");
                for (uri, errors) in map_errors_to_docs(compilation_uri, &compilation.0.errors) {
                    if !docs_with_errors.insert(uri.clone()) {
                        // We already published diagnostics for this document for
                        // a different compilation.
                        // When the same document is included in multiple compilations,
                        // only report the errors for one of them, the goal being
                        // a less confusing user experience.
                        continue;
                    }

                    self.publish_diagnostics_for_doc(state, &uri, errors);
                }
            }

            // Clear errors from any documents that previously had errors
            for uri in last_docs_with_errors.difference(&docs_with_errors) {
                self.publish_diagnostics_for_doc(state, uri, vec![]);
            }
        });

        self.documents_with_errors = docs_with_errors;
    }

    fn publish_diagnostics_for_doc(&self, state: &CompilationState, uri: &str, errors: Vec<Error>) {
        let version = state.open_documents.get(uri).map(|d| d.version);
        trace!(
            "publishing diagnostics for {uri} {version:?}): {} errors",
            errors.len()
        );
        (self.diagnostics_receiver)(DiagnosticUpdate {
            uri: uri.into(),
            version,
            errors,
        });
    }

    fn apply_configuration(&mut self, configuration: WorkspaceConfigurationUpdate) -> bool {
        let mut need_recompile = false;

        if let Some(package_type) = configuration.package_type {
            need_recompile |= self.configuration.package_type != package_type;
            self.configuration.package_type = package_type;
        }

        if let Some(target_profile) = configuration.target_profile {
            need_recompile |= self.configuration.target_profile != target_profile;
            self.configuration.target_profile = target_profile;
        }

        // Possible optimization: some projects will have overrides for these configurations,
        // so workspace updates won't impact them. We could exclude those projects
        // from recompilation, but we don't right now.
        trace!("need_recompile after configuration update: {need_recompile}");
        need_recompile
    }

    /// Recompiles the currently known documents with
    /// the current configuration. Publishes updated
    /// diagnostics for all documents.
    fn recompile_all(&mut self) {
        self.with_state_mut(|state| {
            for compilation in state.compilations.values_mut() {
                let configuration = merge_configurations(compilation.1, self.configuration);
                compilation
                    .0
                    .recompile(configuration.package_type, configuration.target_profile);
            }
        });

        self.publish_diagnostics();
    }

    /// Borrows the compilation state immutably and invokes `f`.
    /// Warning: This function is not reentrant. For dynamic borrow safety,
    /// don't call `with_state` from within `with_state` or `with_state_mut`.
    /// Use a direct reference to the state instead.
    /// This function may also not be async since holding a borrow across
    /// `await` points will interfere with other borrowers.
    fn with_state<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&CompilationState) -> T,
    {
        let state = self.state.borrow();
        f(&state)
    }

    /// Borrows the compilation state immutably and invokes `f`.
    /// Warning: This function is not reentrant.  For dynamic borrow safety,
    /// don't call `with_state_mut` from within `with_state` or `with_state_mut`.
    /// Use a direct reference to the state instead.
    /// This function may also not be async since holding a borrow across
    /// `await` points will interfere with other borrowers.
    fn with_state_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut CompilationState) -> T,
    {
        let mut state = self.state.borrow_mut();
        f(&mut state)
    }
}

impl CompilationState {
    pub(crate) fn get_compilation(&self, uri: &str) -> Option<&Compilation> {
        let Some(compilation_uri) = &self
            .open_documents
            .get(uri)
            .as_ref()
            .map(|x| x.compilation.clone())
        else {
            return None;
        };

        trace!("document: {uri} compilation_uri: {compilation_uri}");

        Some(&self.compilations.get(compilation_uri).unwrap_or_else(|| {
            panic!("document associated with compilation that hasn't been initialized ({compilation_uri})" ,)
        }).0)
    }
}

fn map_errors_to_docs(
    compilation_uri: &Arc<str>,
    errors: &Vec<Error>,
) -> FxHashMap<Arc<str>, Vec<Error>> {
    let mut map = FxHashMap::default();

    for err in errors {
        // Use the compilation_uri as a location for span-less errors
        let doc = err
            .labels()
            .into_iter()
            .flatten()
            .next()
            .map_or(compilation_uri, |l| {
                let (source, _) = err.resolve_span(l.inner());
                &source.name
            });

        map.entry(doc.clone())
            .or_insert_with(Vec::new)
            .push(err.clone());
    }

    map
}

/// Merges workspace configuration with any compilation-specific overrides.
fn merge_configurations(
    compilation_overrides: PartialConfiguration,
    workspace_scope: Configuration,
) -> Configuration {
    Configuration {
        target_profile: compilation_overrides
            .target_profile
            .unwrap_or(workspace_scope.target_profile),
        package_type: compilation_overrides
            .package_type
            .unwrap_or(workspace_scope.package_type),
    }
}

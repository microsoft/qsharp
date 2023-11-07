use crate::{set_indentation, RuntimeCapability};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{ItemId, LocalItemId, PackageId, PackageStore};

use indenter::indented;
use rustc_hash::FxHashSet;

use std::{
    fmt::{Display, Formatter, Result, Write},
    vec::Vec,
};

pub struct StoreRtProps(IndexMap<PackageId, PackageRtProps>);

#[derive(Debug)]
pub struct PackageRtProps {
    pub items: IndexMap<LocalItemId, Option<ItemRtProps>>,
    pub blocks: IndexMap<LocalItemId, Option<InnerElmtRtProps>>,
    pub stmts: IndexMap<LocalItemId, Option<InnerElmtRtProps>>,
    pub exprs: IndexMap<LocalItemId, Option<InnerElmtRtProps>>,
    pub pats: IndexMap<LocalItemId, Option<PatRtProps>>,
}

impl Default for PackageRtProps {
    fn default() -> Self {
        Self {
            items: IndexMap::new(),
            blocks: IndexMap::new(),
            stmts: IndexMap::new(),
            exprs: IndexMap::new(),
            pats: IndexMap::new(),
        }
    }
}

impl Display for PackageRtProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Items:")?;
        for (id, item) in self.items.iter() {
            let mut indent = set_indentation(indented(f), 1);
            write!(indent, "\nLocalItemId {id}: ")?;
            match item {
                None => {
                    _ = write!(f, "None");
                }
                Some(item_rt_props) => {
                    let mut indent = set_indentation(indented(f), 2);
                    write!(indent, "\n{item_rt_props}")?;
                }
            }
        }

        write!(f, "\nBlocks:")?;
        for (id, block) in self.blocks.iter() {
            let mut indent = set_indentation(indented(f), 1);
            write!(indent, "\nBlockId {id}: ")?;
            match block {
                None => {
                    _ = write!(f, "None");
                }
                Some(inner_elmt_props) => {
                    let mut indent = set_indentation(indented(f), 2);
                    write!(indent, "\n{inner_elmt_props}")?;
                }
            }
        }
        Ok(())
    }
}

impl PackageRtProps {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
pub enum ItemRtProps {
    NonCallable,
    Callable(CallableRtProps),
}

impl Display for ItemRtProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            ItemRtProps::NonCallable => write!(f, "Non-Callable")?,
            ItemRtProps::Callable(callable_rt_props) => {
                write!(f, "Callable Runtime Properties: {callable_rt_props}")?
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CallableRtProps {
    pub apps_table: AppsTable,
}

impl Display for CallableRtProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Callable Runtime Properties:")?;
        let mut indent = set_indentation(indented(f), 1);
        write!(indent, "\nApplications Table: {}", self.apps_table)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum InnerElmtRtProps {
    AppDependent(AppsTable),
    AppIndependent(ComputeKind),
}

impl Display for InnerElmtRtProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            InnerElmtRtProps::AppDependent(apps_table) => {
                write!(f, "Application Dependent: {apps_table}")?
            }
            InnerElmtRtProps::AppIndependent(compute_kind) => {
                write!(f, "Application Independent: {compute_kind}")?
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParamIdx(usize);

#[derive(Debug)]
pub enum PatRtProps {
    Local,
    CallableParam(ItemId, ParamIdx),
}

impl Display for PatRtProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            PatRtProps::Local => write!(f, "Local")?,
            PatRtProps::CallableParam(item_id, param_idx) => {
                write!(f, "Callable Parameter:")?;
                let mut indent = set_indentation(indented(f), 1);
                write!(indent, "\nItem ID: {item_id}")?;
                write!(indent, "\nParameter Index: {param_idx:?}")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppIdx(usize);

#[derive(Debug)]
pub struct AppsTable {
    // CONSIDER (cesarzc): whether this has to be wrapped in an option or can be just `RtProps`.
    apps: Vec<Option<ComputeKind>>,
}

impl AppsTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            apps: Vec::with_capacity(capacity),
        }
    }

    pub fn get(&self, index: AppIdx) -> Option<&ComputeKind> {
        self.apps[index.0].as_ref()
    }

    pub fn get_mut(&mut self, index: AppIdx) -> Option<&mut ComputeKind> {
        self.apps[index.0].as_mut()
    }
}

impl Display for AppsTable {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Applications Table:")?;
        let mut indent = set_indentation(indented(f), 1);
        for (idx, app) in self.apps.iter().enumerate() {
            let app_str = match app {
                None => "None".to_string(),
                Some(compute_kind) => format!("{compute_kind}"),
            };
            write!(indent, "\n[{idx:b}] -> {app_str}]")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ComputeKind {
    Classical,
    Hybrid,
    Quantum(QuantumCompute),
}

impl Display for ComputeKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            ComputeKind::Classical => write!(f, "Classical")?,
            ComputeKind::Hybrid => write!(f, "Hybrid")?,
            ComputeKind::Quantum(quantum_compute) => write!(f, "{quantum_compute}")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum QuantumSource {
    ItemId,
    BlockId,
    StmtId,
    ExprId,
    PatId,
}

#[derive(Debug)]
pub struct QuantumCompute {
    pub caps: FxHashSet<RuntimeCapability>,
    pub source_trace: Vec<QuantumSource>, // N.B. (cesarzc): To get good error messages.
}

impl Display for QuantumCompute {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "QuantumCompute:")?;
        let mut indent = set_indentation(indented(f), 1);
        if self.caps.is_empty() {
            write!(indent, "\nCapabilities: <empty>")?;
        } else {
            write!(indent, "\nCapabilities: {{")?;
            for cap in &self.caps {
                indent = set_indentation(indent, 2);
                write!(indent, "\n{cap:?}")?;
            }
            indent = set_indentation(indent, 1);
            write!(indent, "\n}}")?;
        }

        let mut indent = set_indentation(indented(f), 1);
        write!(indent, "\nSourceTrace:")?;
        for src in self.source_trace.iter() {
            indent = set_indentation(indent, 2);
            write!(indent, "\n{src:?}")?; // TODO (cesarzc): Implement non-debug display, maybe?.
        }
        indent = set_indentation(indent, 1);
        Ok(())
    }
}

pub struct Analyzer<'a> {
    package_store: &'a PackageStore,
    store_rt_props: StoreRtProps,
}

impl<'a> Analyzer<'a> {
    pub fn new(package_store: &'a PackageStore) -> Self {
        let mut packages_rt_props = IndexMap::new();
        for (package_id, _) in package_store.0.iter() {
            packages_rt_props.insert(package_id, PackageRtProps::new())
        }
        Self {
            package_store,
            store_rt_props: StoreRtProps(packages_rt_props),
        }
    }

    pub fn run<'b>(&'b mut self) -> &'b StoreRtProps {
        &self.store_rt_props
    }
}

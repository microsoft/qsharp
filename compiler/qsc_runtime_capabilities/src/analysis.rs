use crate::{set_indentation, RuntimeCapability};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{ItemId, LocalItemId};

use indenter::indented;
use rustc_hash::FxHashSet;

use std::{
    fmt::{Display, Formatter, Result, Write},
    ops::Deref,
    vec::Vec,
};

#[derive(Debug)]
pub struct StoreRtProps {
    pub items: IndexMap<LocalItemId, Option<ItemRtProps>>,
    pub blocks: IndexMap<LocalItemId, Option<InnerElmtRtProps>>,
    pub stmts: IndexMap<LocalItemId, Option<InnerElmtRtProps>>,
    pub exprs: IndexMap<LocalItemId, Option<InnerElmtRtProps>>,
    pub pats: IndexMap<LocalItemId, Option<PatRtProps>>,
}

#[derive(Debug)]
pub enum ItemRtProps {
    NonCallable,
    Callable(CallableRtProps),
}

#[derive(Debug)]
pub struct CallableRtProps {
    pub apps_table: AppsTable,
}

#[derive(Debug)]
pub enum InnerElmtRtProps {
    AppDependent(AppsTable),
    AppIndependent(Compute),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParamIdx(usize);

#[derive(Debug)]
pub enum PatRtProps {
    Local,
    CallableParam(ItemId, ParamIdx),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppIdx(usize);

#[derive(Debug)]
pub struct AppsTable {
    // CONSIDER (cesarzc): whether this has to be wrapped in an option or can be just `RtProps`.
    apps: Vec<Option<Compute>>,
}

impl AppsTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            apps: Vec::with_capacity(capacity),
        }
    }

    pub fn get(&self, index: AppIdx) -> Option<&Compute> {
        self.apps[index.0].as_ref()
    }

    pub fn get_mut(&mut self, index: AppIdx) -> Option<&mut Compute> {
        self.apps[index.0].as_mut()
    }
}

impl Display for AppsTable {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "ApplicationsTable:")?;
        let mut indent = set_indentation(indented(f), 1);
        for (idx, app) in self.apps.iter().enumerate() {
            let app_str = match app {
                None => "None".to_string(),
                Some(compute) => format!("{compute:?}"), // TODO (cesarzc): Implemnt non-debug display.
            };
            write!(indent, "\n[{idx:b}] -> {app_str}]")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Compute {
    Classical,
    Quantum(QuantumCompute),
}

#[derive(Debug)]
pub enum QuantumSouce {
    ItemId,
    BlockId,
    StmtId,
    ExprId,
    PatId,
}

#[derive(Debug)]
pub struct QuantumCompute {
    pub caps: FxHashSet<RuntimeCapability>,
    pub source_trace: Vec<QuantumSouce>, // N.B. (cesarzc): To get good error messages.
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
            write!(indent, "\nCapabilities: }}")?;
        }

        let mut indent = set_indentation(indented(f), 1);
        write!(indent, "\nSourceTrace: {{")?;
        for src in self.source_trace.iter() {
            indent = set_indentation(indent, 2);
            write!(indent, "\n{src:?}")?; // TODO (cesarzc): Implement non-debug display, maybe?.
        }
        indent = set_indentation(indent, 1);
        write!(indent, "\nCapabilities: }}")?;
        Ok(())
    }
}

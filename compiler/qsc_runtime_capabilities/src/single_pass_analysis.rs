use crate::{set_indentation, RuntimeCapability};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{
    BlockId, CallableDecl, ExprId, Item, ItemId, ItemKind, LocalItemId, Package, PackageId,
    PackageStore, PatId, StmtId,
};

use indenter::indented;
use rustc_hash::{FxHashMap, FxHashSet};

use std::{
    fmt::{Display, Formatter, Result, Write},
    fs::File,
    io::Write as IoWrite,
    vec::Vec,
};

#[derive(Debug)]
pub struct StoreComputeProps(IndexMap<PackageId, PackageComputeProps>);

impl StoreComputeProps {
    pub fn incorporate_partial_compute_props(
        &mut self,
        store_partial_compute_props: &mut StorePartialComputeProps,
    ) {
        for (package_id, package_partial_compute_props) in store_partial_compute_props.0.iter_mut()
        {
            let package_compute_props: &mut PackageComputeProps = match self.0.get_mut(*package_id)
            {
                None => {
                    self.0.insert(*package_id, PackageComputeProps::default());
                    self.0
                        .get_mut(*package_id)
                        .expect("Package compute properties should exist")
                }
                Some(p) => p,
            };

            package_compute_props.incorporate_partial_compute_props(package_partial_compute_props);
        }
    }

    pub fn has_item(&self, package_id: PackageId, item_id: LocalItemId) -> bool {
        if let Some(package) = self.0.get(package_id) {
            return package.items.contains_key(item_id);
        }
        false
    }

    pub fn persist(&self) {
        for (package_id, package) in self.0.iter() {
            let filename = format!("dbg/rca.package{package_id}.txt");
            let mut package_file = File::create(filename).expect("File could be created");
            let package_string = format!("{package}");
            write!(package_file, "{package_string}").expect("Writing to file should succeed.");
        }
    }
}

#[derive(Debug)]
pub struct PackageComputeProps {
    pub items: IndexMap<LocalItemId, ItemComputeProps>,
    pub blocks: IndexMap<BlockId, InnerElmtComputeProps>,
    pub stmts: IndexMap<StmtId, InnerElmtComputeProps>,
    pub exprs: IndexMap<ExprId, InnerElmtComputeProps>,
    pub pats: IndexMap<PatId, PatComputeProps>,
}

impl Default for PackageComputeProps {
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

impl Display for PackageComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Items:")?;
        for (id, item) in self.items.iter() {
            let mut indent = set_indentation(indented(f), 1);
            write!(indent, "\nLocal Item ID {id}: ")?;
            let mut indent = set_indentation(indented(f), 2);
            write!(indent, "\n{item}")?;
        }

        write!(f, "\nBlocks:")?;
        for (id, block) in self.blocks.iter() {
            let mut indent = set_indentation(indented(f), 1);
            write!(indent, "\nBlock ID {id}: ")?;
            let mut indent = set_indentation(indented(f), 2);
            write!(indent, "\n{block}")?;
        }

        write!(f, "\nStatements:")?;
        for (id, stmt) in self.stmts.iter() {
            let mut indent = set_indentation(indented(f), 1);
            write!(indent, "\nStatement ID {id}: ")?;
            let mut indent = set_indentation(indented(f), 2);
            write!(indent, "\n{stmt}")?;
        }

        write!(f, "\nExpressions:")?;
        for (id, expr) in self.exprs.iter() {
            let mut indent = set_indentation(indented(f), 1);
            write!(indent, "\nExpression ID {id}: ")?;
            let mut indent = set_indentation(indented(f), 2);
            write!(indent, "\n{expr}")?;
        }

        write!(f, "\nPatterns:")?;
        for (id, pat) in self.pats.iter() {
            let mut indent = set_indentation(indented(f), 1);
            write!(indent, "\nPattern ID {id}: ")?;
            let mut indent = set_indentation(indented(f), 2);
            write!(indent, "\n{pat}")?;
        }
        Ok(())
    }
}

impl PackageComputeProps {
    pub fn incorporate_partial_compute_props(
        &mut self,
        partial_compute_props: &mut PackagePartialComputeProps,
    ) {
        partial_compute_props
            .items
            .drain()
            .for_each(|(item_id, item)| self.items.insert(item_id, item));

        partial_compute_props
            .blocks
            .drain()
            .for_each(|(block_id, block)| self.blocks.insert(block_id, block));

        partial_compute_props
            .stmts
            .drain()
            .for_each(|(stmt_id, stmt)| self.stmts.insert(stmt_id, stmt));

        partial_compute_props
            .exprs
            .drain()
            .for_each(|(expr_id, expr)| self.exprs.insert(expr_id, expr));

        partial_compute_props
            .pats
            .drain()
            .for_each(|(pat_id, pat)| self.pats.insert(pat_id, pat));
    }
}

#[derive(Debug)]
pub enum ItemComputeProps {
    NonCallable,
    Callable(CallableComputeProps),
}

impl Display for ItemComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            ItemComputeProps::NonCallable => write!(f, "Non-Callable")?,
            ItemComputeProps::Callable(callable_rt_props) => write!(f, "{callable_rt_props}")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CallableComputeProps {
    pub apps: AppsTbl,
}

impl Display for CallableComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Callable Runtime Properties:")?;
        let mut indent = set_indentation(indented(f), 1);
        write!(indent, "\n{}", self.apps)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum InnerElmtComputeProps {
    AppDependent(AppsTbl),
    AppIndependent(ComputeProps),
}

impl Display for InnerElmtComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            InnerElmtComputeProps::AppDependent(apps_table) => {
                write!(f, "Application Dependent: {apps_table}")?
            }
            InnerElmtComputeProps::AppIndependent(compute_kind) => {
                write!(f, "Application Independent: {compute_kind}")?
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParamIdx(usize);

#[derive(Debug)]
pub enum PatComputeProps {
    Local,
    CallableParam(ItemId, ParamIdx),
}

impl Display for PatComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            PatComputeProps::Local => write!(f, "Local")?,
            PatComputeProps::CallableParam(item_id, param_idx) => {
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

impl AppIdx {
    pub fn map_to_compute_kind_vector(&self, input_param_count: usize) -> Vec<ComputeKind> {
        let mut params_compute_kind = Vec::new();
        for param_idx in 0..input_param_count {
            let mask = 1 << param_idx;
            let compute_kind = if self.0 & mask == 0 {
                ComputeKind::Static
            } else {
                ComputeKind::Dynamic
            };
            params_compute_kind.push(compute_kind);
        }
        params_compute_kind
    }
}

#[derive(Clone, Debug)]
pub struct AppsTbl {
    // N.B. (cesarzc): Will probably be only used to assert compatibility when using it.
    pub input_param_count: usize,
    // N.B. (cesarzc): Hide the vector to provide a good interface to access applications (possibly
    // by providing a get that takes a vector of `ComputeKind`).
    apps: Vec<ComputeProps>,
}

impl AppsTbl {
    pub fn new(input_param_count: usize) -> Self {
        Self {
            input_param_count,
            apps: Vec::new(),
        }
    }

    // TODO (cesarzc): Implement a get that takes a vector of `ComputeKind` where each element maps
    // to the compute kind of the input parameter.

    pub fn get(&self, index: AppIdx) -> Option<&ComputeProps> {
        self.apps.get(index.0)
    }

    pub fn get_mut(&mut self, index: AppIdx) -> Option<&mut ComputeProps> {
        self.apps.get_mut(index.0)
    }

    pub fn push(&mut self, app: ComputeProps) {
        self.apps.push(app);
    }
}

impl Display for AppsTbl {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Applications Table ({} input parameters):",
            self.input_param_count
        )?;
        let mut indent = set_indentation(indented(f), 1);
        for (idx, app) in self.apps.iter().enumerate() {
            write!(indent, "\n[{idx:#010b}] -> {app}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ComputeProps {
    pub rt_caps: FxHashSet<RuntimeCapability>,
    // N.B. (cesarzc): To get good error messages, maybe quantum source needs expansion and link to compute props.
    pub quantum_sources: Vec<QuantumSource>,
}

impl Display for ComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let compute_kind = self.compute_kind();
        write!(f, "Compute Properties ({compute_kind:?}):")?;
        let mut indent = set_indentation(indented(f), 1);
        if self.rt_caps.is_empty() {
            write!(indent, "\nRuntime Capabilities: <empty>")?;
        } else {
            write!(indent, "\nRuntime Capabilities: {{")?;
            for cap in &self.rt_caps {
                indent = set_indentation(indent, 2);
                write!(indent, "\n{cap:?}")?;
            }
            indent = set_indentation(indent, 1);
            write!(indent, "\n}}")?;
        }

        let mut indent = set_indentation(indented(f), 1);
        if self.quantum_sources.is_empty() {
            write!(indent, "\nQuantum Sources: <empty>")?;
        } else {
            write!(indent, "\nQuantum Sources:")?;
            for src in self.quantum_sources.iter() {
                indent = set_indentation(indent, 2);
                write!(indent, "\n{src:?}")?; // TODO (cesarzc): Implement non-debug display, maybe?.
            }
        }

        Ok(())
    }
}

impl ComputeProps {
    pub fn is_quantum_source(&self) -> bool {
        !self.quantum_sources.is_empty()
    }

    pub fn compute_kind(&self) -> ComputeKind {
        if self.rt_caps.is_empty() {
            ComputeKind::Static
        } else {
            ComputeKind::Dynamic
        }
    }
}

#[derive(Debug)]
pub enum ComputeKind {
    Static,
    Dynamic,
}

#[derive(Clone, Debug)]
pub enum QuantumSource {
    Intrinsic,
    ItemId,
    BlockId,
    StmtId,
    ExprId,
    PatId,
}

#[derive(Debug)]
pub struct StorePartialComputeProps(FxHashMap<PackageId, PackagePartialComputeProps>);

#[derive(Debug, Default)]
pub struct PackagePartialComputeProps {
    pub items: FxHashMap<LocalItemId, ItemComputeProps>,
    pub blocks: FxHashMap<BlockId, InnerElmtComputeProps>,
    pub stmts: FxHashMap<StmtId, InnerElmtComputeProps>,
    pub exprs: FxHashMap<ExprId, InnerElmtComputeProps>,
    pub pats: FxHashMap<PatId, PatComputeProps>,
}

pub struct SinglePassAnalyzer;

impl SinglePassAnalyzer {
    pub fn run(package_store: &PackageStore) -> StoreComputeProps {
        let mut store_compute_props = StoreComputeProps(IndexMap::new());

        //
        for (package_id, package) in package_store.0.iter() {
            for (item_id, item) in package.items.iter() {
                if !store_compute_props.has_item(package_id, item_id) {
                    let mut store_partial_compute_props = Self::analyze_item_compute_props(
                        package_id,
                        item_id,
                        item,
                        &store_compute_props,
                    );
                    store_compute_props
                        .incorporate_partial_compute_props(&mut store_partial_compute_props);
                }
            }
        }
        store_compute_props
    }

    fn analyze_callable_compute_props(
        package_id: PackageId,
        item_id: LocalItemId,
        item: &Item,
        store_compute_props: &StoreComputeProps,
    ) -> StorePartialComputeProps {
        let callable_compute_props = CallableComputeProps {
            apps: AppsTbl::new(0),
        };
        let mut package_partial_compute_props = PackagePartialComputeProps::default();
        package_partial_compute_props
            .items
            .insert(item_id, ItemComputeProps::Callable(callable_compute_props));
        let mut store_partial_compute_props = StorePartialComputeProps(FxHashMap::default());
        store_partial_compute_props
            .0
            .insert(package_id, package_partial_compute_props);
        store_partial_compute_props
    }

    fn analyze_item_compute_props(
        package_id: PackageId,
        item_id: LocalItemId,
        item: &Item,
        store_compute_props: &StoreComputeProps,
    ) -> StorePartialComputeProps {
        match item.kind {
            ItemKind::Namespace(..) | ItemKind::Ty(..) => {
                Self::create_non_callable_item_partial_compute_props(package_id, item_id)
            }
            _ => {
                Self::analyze_callable_compute_props(package_id, item_id, item, store_compute_props)
            }
        }
    }

    fn create_non_callable_item_partial_compute_props(
        package_id: PackageId,
        item_id: LocalItemId,
    ) -> StorePartialComputeProps {
        let mut package_partial_compute_props = PackagePartialComputeProps::default();
        package_partial_compute_props
            .items
            .insert(item_id, ItemComputeProps::NonCallable);
        let mut store_partial_compute_props = StorePartialComputeProps(FxHashMap::default());
        store_partial_compute_props
            .0
            .insert(package_id, package_partial_compute_props);
        store_partial_compute_props
    }
}

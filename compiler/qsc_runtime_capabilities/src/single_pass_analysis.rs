use crate::{set_indentation, RuntimeCapability};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        BlockId, CallableDecl, CallableKind, Expr, ExprId, ExprKind, Item, ItemId, ItemKind,
        LocalItemId, NodeId, PackageId, PackageStore, Pat, PatId, PatKind, Res, SpecBody, SpecGen,
        Stmt, StmtId, StmtKind,
    },
    ty::{Prim, Ty},
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
    pub fn incorporate_scratch(&mut self, store_scratch: &mut StoreScratch) {
        for (package_id, package_scratch) in store_scratch.0.iter_mut() {
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

            package_compute_props.incorporate_scratch(package_scratch);
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
    pub blocks: IndexMap<BlockId, ElmntComputeProps>,
    pub stmts: IndexMap<StmtId, ElmntComputeProps>,
    pub exprs: IndexMap<ExprId, ExprComputeProps>,
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
    pub fn incorporate_scratch(&mut self, package_scratch: &mut PackageScratch) {
        package_scratch
            .items
            .drain()
            .for_each(|(item_id, item)| self.items.insert(item_id, item));

        package_scratch
            .blocks
            .drain()
            .for_each(|(block_id, block)| self.blocks.insert(block_id, block));

        package_scratch
            .stmts
            .drain()
            .for_each(|(stmt_id, stmt)| self.stmts.insert(stmt_id, stmt));

        package_scratch
            .exprs
            .drain()
            .for_each(|(expr_id, expr)| self.exprs.insert(expr_id, expr));

        package_scratch
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
            ItemComputeProps::Callable(callable_rt_props) => write!(f, "{callable_rt_props}")?,
            ItemComputeProps::NonCallable => write!(f, "Non-Callable")?,
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
        write!(f, "Callable Compute Properties:")?;
        let mut indent = set_indentation(indented(f), 1);
        write!(indent, "\n{}", self.apps)?;
        Ok(())
    }
}

impl ElmntComputeProps {
    pub fn from_expr_compute_props(expr_compute_props: &ExprComputeProps) -> Self {
        match expr_compute_props {
            ExprComputeProps::Elmnt(elmnt) => elmnt.clone(),
            ExprComputeProps::Global(_, _) | ExprComputeProps::Local(_, _, _) => {
                ElmntComputeProps::AppIndependent(ComputeProps::default())
            }
            ExprComputeProps::Unsupported => ElmntComputeProps::Unsupported,
        }
    }
}

#[derive(Debug)]
pub struct StmtComputeProps {}

#[derive(Clone, Debug)]
pub enum ExprComputeProps {
    Elmnt(ElmntComputeProps),
    Global(PackageId, LocalItemId),
    Local(NodeId, PackageId, PatId),
    Unsupported, // TODO (cesarzc): This should eventually be removed but keep it for now while doing implementation.
}

impl Display for ExprComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            ExprComputeProps::Elmnt(concrete) => write!(f, "Concrete: {concrete}")?,
            ExprComputeProps::Global(package_id, item_id) => {
                write!(f, "Global ({package_id:?} | {item_id:?})")?
            }
            ExprComputeProps::Local(node_id, package_id, pat_id) => {
                write!(f, "Input Param ({node_id} | {package_id:?} | {pat_id:?})")?
            }
            ExprComputeProps::Unsupported => write!(f, "Unsupported")?,
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParamIdx(usize);

#[derive(Debug)]
pub enum PatComputeProps {
    LocalGroup, // TODO (cesarzc): Include PatIds.
    LocalNode(NodeId, ExprComputeProps),
    InputParamGroup, // TODO (cesarzc): Include PatIds.
    InputParamNode(NodeId, PackageId, LocalItemId, ParamIdx, Ty),
    Unsupported, // TODO (cesarzc): This is temporary and should be removed once the implementation is complete.
}

impl Display for PatComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            PatComputeProps::LocalGroup => write!(f, "Local Group")?,
            PatComputeProps::LocalNode(node_id, expr_compute_props) => {
                write!(f, "Local Node ({node_id}): {expr_compute_props}")?
            }
            PatComputeProps::InputParamGroup => write!(f, "Input Param Group")?,
            PatComputeProps::InputParamNode(node_id, package_id, local_item_id, param_idx, ty) => {
                write!(f, "Callable Parameter (Node ID {node_id}):")?;
                let mut indent = set_indentation(indented(f), 1);
                write!(indent, "\nPackage ID: {package_id}")?;
                write!(indent, "\nLocal Item ID: {local_item_id}")?;
                write!(indent, "\nParameter Index: {param_idx:?}")?;
                write!(indent, "\nType: {ty}")?;
            }
            PatComputeProps::Unsupported => write!(f, "Unsupported")?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum ElmntComputeProps {
    AppDependent(AppsTbl),
    AppIndependent(ComputeProps),
    Unsupported, // TODO (cesarzc): This should eventually be removed but keep it for now while doing implementation.
}

impl Display for ElmntComputeProps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match &self {
            ElmntComputeProps::AppDependent(apps_table) => {
                write!(f, "Application Dependent: {apps_table}")?
            }
            ElmntComputeProps::AppIndependent(compute_kind) => {
                write!(f, "Application Independent: {compute_kind}")?
            }
            ElmntComputeProps::Unsupported => write!(f, "Unsupported")?,
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppIdx(usize);

impl AppIdx {
    pub fn represents_all_static_params(&self) -> bool {
        self.0 == 0
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
            // use with_capacity().
            apps: Vec::new(),
        }
    }

    pub fn get(&self, index: AppIdx) -> Option<&ComputeProps> {
        self.apps.get(index.0)
    }

    pub fn get_mut(&mut self, index: AppIdx) -> Option<&mut ComputeProps> {
        self.apps.get_mut(index.0)
    }

    pub fn max(&self) -> usize {
        2i32.pow(self.input_param_count as u32) as usize
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

#[derive(Clone, Debug, Default)]
pub struct ComputeProps {
    pub rt_caps: FxHashSet<RuntimeCapability>,
    // N.B. (cesarzc): To get good error messages, maybe quantum source needs expansion and link to compute props.
    // TODO (cesarzc): Should probably just be one quantum source.
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

#[derive(Clone, Debug)]
pub enum ComputeKind {
    Static,
    Dynamic,
}

#[derive(Clone, Debug)]
// TODO (cesarzc): Should probably include package ID as well.
pub enum QuantumSource {
    Intrinsic,
    ItemId,
    BlockId,
    StmtId,
    ExprId,
    PatId,
}

#[derive(Debug)]
struct NodeMap(FxHashMap<NodeId, Node>);

impl NodeMap {
    fn from_callable_input(callable: &CallableDecl, package_pats: &IndexMap<PatId, Pat>) -> Self {
        let input_pat = package_pats
            .get(callable.input)
            .expect("Callable input pattern should exist");

        fn from_pat(pat: &Pat, pats: &IndexMap<PatId, Pat>) -> Vec<(NodeId, PatId, Ty)> {
            match &pat.kind {
                PatKind::Bind(ident) => vec![(ident.id, pat.id, pat.ty.clone())],
                PatKind::Tuple(tuple_pats) => {
                    let mut tuple_params = Vec::<(NodeId, PatId, Ty)>::new();
                    for tuple_item_pat_id in tuple_pats {
                        let tuple_item_pat = pats
                            .get(*tuple_item_pat_id)
                            .expect("`Pattern` should exist");
                        let mut tuple_item_params = from_pat(tuple_item_pat, pats);
                        tuple_params.append(&mut tuple_item_params);
                    }
                    tuple_params
                }
                _ => panic!("Only callable parameter patterns are expected"),
            }
        }

        let mut node_map = FxHashMap::<NodeId, Node>::default();
        for (idx, (node_id, pat_id, ty)) in from_pat(input_pat, package_pats).iter().enumerate() {
            let node = Node {
                id: *node_id,
                pat_id: *pat_id,
                kind: NodeKind::InputParam(InputParamIdx(idx), ty.clone()),
            };
            node_map.insert(*node_id, node);
        }

        Self(node_map)
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub pat_id: PatId,
    pub kind: NodeKind,
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    InputParam(InputParamIdx, Ty),
    Local,
}

#[derive(Clone, Debug)]
pub struct InputParamIdx(usize);

#[derive(Debug)]
struct InputParamsMap(FxHashMap<NodeId, (PatId, InputParamIdx, Ty)>);

impl InputParamsMap {
    fn from_callable(callable: &CallableDecl, package_pats: &IndexMap<PatId, Pat>) -> Self {
        let input_pat = package_pats
            .get(callable.input)
            .expect("Callable input pattern should exist");

        fn from_pat(pat: &Pat, pats: &IndexMap<PatId, Pat>) -> Vec<(NodeId, PatId, Ty)> {
            match &pat.kind {
                PatKind::Bind(ident) => vec![(ident.id, pat.id, pat.ty.clone())],
                PatKind::Tuple(tuple_pats) => {
                    let mut tuple_params = Vec::<(NodeId, PatId, Ty)>::new();
                    for tuple_item_pat_id in tuple_pats {
                        let tuple_item_pat = pats
                            .get(*tuple_item_pat_id)
                            .expect("`Pattern` should exist");
                        let mut tuple_item_params = from_pat(tuple_item_pat, pats);
                        tuple_params.append(&mut tuple_item_params);
                    }
                    tuple_params
                }
                _ => panic!("Only callable parameter patterns are expected"),
            }
        }

        let mut input_params = FxHashMap::<NodeId, (PatId, InputParamIdx, Ty)>::default();
        for (idx, (node_id, pat_id, ty)) in from_pat(input_pat, package_pats).iter().enumerate() {
            input_params.insert(*node_id, (*pat_id, InputParamIdx(idx), ty.clone()));
        }
        InputParamsMap(input_params)
    }
}

#[derive(Debug)]
struct InputParamsApp {
    pub idx: AppIdx,
    pub params_compute_kind: FxHashMap<NodeId, (Node, ComputeKind)>,
}

impl InputParamsApp {
    pub fn from_app_idx(app_idx: AppIdx, input_params: &NodeMap) -> Self {
        // The application index must be smaller than 2^n where n is the number of input parameters.
        assert!((app_idx.0 as i32) < 2i32.pow(input_params.0.len() as u32));
        let mut params_compute_kind = FxHashMap::<NodeId, (Node, ComputeKind)>::default();
        for (node_id, node) in input_params.0.iter() {
            if let NodeKind::InputParam(input_param_idx, _) = &node.kind {
                let mask = 1 << input_param_idx.0;
                let compute_kind = if app_idx.0 & mask == 0 {
                    ComputeKind::Static
                } else {
                    ComputeKind::Dynamic
                };
                params_compute_kind.insert(*node_id, (node.clone(), compute_kind));
            } else {
                panic!("Only input parameters are expected");
            }
        }
        Self {
            idx: app_idx,
            params_compute_kind,
        }
    }
}

#[derive(Debug, Default)]
pub struct StoreScratch(FxHashMap<PackageId, PackageScratch>);

impl StoreScratch {
    pub fn get_expr(&self, package_id: &PackageId, expr_id: &ExprId) -> Option<&ExprComputeProps> {
        self.0
            .get(package_id)
            .and_then(|package| package.exprs.get(expr_id))
    }

    pub fn get_item(
        &self,
        package_id: &PackageId,
        item_id: &LocalItemId,
    ) -> Option<&ItemComputeProps> {
        self.0
            .get(package_id)
            .and_then(|package| package.items.get(item_id))
    }

    pub fn get_pat(&self, package_id: &PackageId, pat_id: &PatId) -> Option<&PatComputeProps> {
        self.0
            .get(package_id)
            .and_then(|package| package.pats.get(pat_id))
    }

    pub fn get_stmt(&self, package_id: &PackageId, stmt_id: &StmtId) -> Option<&ElmntComputeProps> {
        self.0
            .get(package_id)
            .and_then(|package| package.stmts.get(stmt_id))
    }

    pub fn has_expr(&self, package_id: &PackageId, expr_id: &ExprId) -> bool {
        self.get_expr(package_id, expr_id).is_some()
    }

    pub fn has_pat(&self, package_id: &PackageId, pat_id: &PatId) -> bool {
        self.get_pat(package_id, pat_id).is_some()
    }

    pub fn has_item(&self, package_id: &PackageId, item_id: &LocalItemId) -> bool {
        self.get_item(package_id, item_id).is_some()
    }

    pub fn has_stmt(&self, package_id: &PackageId, stmt_id: &StmtId) -> bool {
        self.get_stmt(package_id, stmt_id).is_some()
    }

    pub fn incorporate_scratch(&mut self, store_scratch: &mut StoreScratch) {
        for (package_id, package_scratch) in store_scratch.0.iter_mut() {
            let self_package_scratch: &mut PackageScratch = match self.0.get_mut(package_id) {
                None => {
                    self.0.insert(*package_id, PackageScratch::default());
                    self.0
                        .get_mut(package_id)
                        .expect("Package compute properties should exist")
                }
                Some(p) => p,
            };

            self_package_scratch.incorporate_scratch(package_scratch);
        }
    }

    pub fn insert_item(
        &mut self,
        package_id: PackageId,
        item_id: LocalItemId,
        item: ItemComputeProps,
    ) {
        let self_package_scratch: &mut PackageScratch = match self.0.get_mut(&package_id) {
            None => {
                self.0.insert(package_id, PackageScratch::default());
                self.0
                    .get_mut(&package_id)
                    .expect("Package compute properties should exist")
            }
            Some(p) => p,
        };

        self_package_scratch.items.insert(item_id, item);
    }

    pub fn insert_expr(&mut self, package_id: PackageId, expr_id: ExprId, expr: ExprComputeProps) {
        let self_package_scratch: &mut PackageScratch = match self.0.get_mut(&package_id) {
            None => {
                self.0.insert(package_id, PackageScratch::default());
                self.0
                    .get_mut(&package_id)
                    .expect("Package compute properties should exist")
            }
            Some(p) => p,
        };

        self_package_scratch.exprs.insert(expr_id, expr);
    }

    pub fn insert_pat(&mut self, package_id: PackageId, pat_id: PatId, pat: PatComputeProps) {
        let self_package_scratch: &mut PackageScratch = match self.0.get_mut(&package_id) {
            None => {
                self.0.insert(package_id, PackageScratch::default());
                self.0
                    .get_mut(&package_id)
                    .expect("Package compute properties should exist")
            }
            Some(p) => p,
        };

        self_package_scratch.pats.insert(pat_id, pat);
    }

    pub fn insert_stmt(&mut self, package_id: PackageId, stmt_id: StmtId, stmt: ElmntComputeProps) {
        let self_package_scratch: &mut PackageScratch = match self.0.get_mut(&package_id) {
            None => {
                self.0.insert(package_id, PackageScratch::default());
                self.0
                    .get_mut(&package_id)
                    .expect("Package compute properties should exist")
            }
            Some(p) => p,
        };

        self_package_scratch.stmts.insert(stmt_id, stmt);
    }

    pub fn with_callable_compute_props(
        package_id: PackageId,
        callable_id: LocalItemId,
        callable_compute_props: CallableComputeProps,
    ) -> Self {
        let mut instance = Self::default();
        let package_scratch =
            PackageScratch::with_callable_compute_props(callable_id, callable_compute_props);
        instance.0.insert(package_id, package_scratch);
        instance
    }

    pub fn with_non_callable_item_compute_props(
        package_id: PackageId,
        item_id: LocalItemId,
    ) -> Self {
        let mut instance = Self::default();
        let package_scratch = PackageScratch::with_non_callable_item_compute_props(item_id);
        instance.0.insert(package_id, package_scratch);
        instance
    }
}

#[derive(Debug, Default)]
pub struct PackageScratch {
    pub items: FxHashMap<LocalItemId, ItemComputeProps>,
    pub blocks: FxHashMap<BlockId, ElmntComputeProps>,
    pub stmts: FxHashMap<StmtId, ElmntComputeProps>,
    pub exprs: FxHashMap<ExprId, ExprComputeProps>,
    pub pats: FxHashMap<PatId, PatComputeProps>,
}

impl PackageScratch {
    pub fn incorporate_scratch(&mut self, package_scratch: &mut PackageScratch) {
        package_scratch.items.drain().for_each(|(item_id, item)| {
            _ = self.items.insert(item_id, item);
        });

        package_scratch
            .blocks
            .drain()
            .for_each(|(block_id, block)| {
                _ = self.blocks.insert(block_id, block);
            });

        package_scratch.stmts.drain().for_each(|(stmt_id, stmt)| {
            _ = self.stmts.insert(stmt_id, stmt);
        });

        package_scratch.exprs.drain().for_each(|(expr_id, expr)| {
            _ = self.exprs.insert(expr_id, expr);
        });

        package_scratch.pats.drain().for_each(|(pat_id, pat)| {
            _ = self.pats.insert(pat_id, pat);
        });
    }

    pub fn with_callable_compute_props(
        callable_id: LocalItemId,
        callable_compute_props: CallableComputeProps,
    ) -> Self {
        let mut instance = Self::default();
        instance.items.insert(
            callable_id,
            ItemComputeProps::Callable(callable_compute_props),
        );
        instance
    }

    pub fn with_non_callable_item_compute_props(item_id: LocalItemId) -> Self {
        let mut instance = Self::default();
        instance
            .items
            .insert(item_id, ItemComputeProps::NonCallable);
        instance
    }
}

pub struct SinglePassAnalyzer;

// TODO (cesarzc): get things out of here. should use clippy.
impl SinglePassAnalyzer {
    pub fn run(package_store: &PackageStore) -> StoreComputeProps {
        let mut store_scratch = StoreScratch::default();

        //
        for (package_id, package) in package_store.0.iter() {
            for (item_id, item) in package.items.iter() {
                if !store_scratch.has_item(&package_id, &item_id) {
                    Self::analyze_item(
                        item,
                        item_id,
                        package_id,
                        package_store,
                        &mut store_scratch,
                    );
                }
            }
        }
        let mut store_compute_props = StoreComputeProps(IndexMap::new());
        store_compute_props.incorporate_scratch(&mut store_scratch);
        store_compute_props
    }

    fn analyze_callable(
        callable: &CallableDecl,
        callable_id: LocalItemId,
        package_id: PackageId,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        // TODO (cesarzc): Set the appropriate values for the callable input parameters.
        if Self::is_callable_intrinsic(callable) {
            let instrinsic_compute_props =
                Self::analyze_intrinsic_callable(callable, package_id, package_store);
            store_scratch.insert_item(
                package_id,
                callable_id,
                ItemComputeProps::Callable(instrinsic_compute_props),
            );
        } else {
            Self::analyze_non_intrinsic_callable(
                callable,
                callable_id,
                package_id,
                package_store,
                store_scratch,
            );
        }
    }

    fn analyze_expr(
        expr_id: ExprId,
        package_id: PackageId,
        _input_params_map: &InputParamsMap,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        if store_scratch.get_expr(&package_id, &expr_id).is_some() {
            return;
        }

        let expr = package_store
            .get_expr(package_id, expr_id)
            .expect("Expression should exist");
        match expr.kind {
            ExprKind::Lit(_) => Self::analyze_expr_lit(expr_id, package_id, store_scratch),
            _ => store_scratch.insert_expr(package_id, expr_id, ExprComputeProps::Unsupported),
        };
    }

    fn analyze_expr_lit(expr_id: ExprId, package_id: PackageId, store_scratch: &mut StoreScratch) {
        let elmt_compute_props = ElmntComputeProps::AppIndependent(ComputeProps::default());
        let compute_props = ExprComputeProps::Elmnt(elmt_compute_props);
        store_scratch.insert_expr(package_id, expr_id, compute_props);
    }

    fn analyze_expr_var(
        expr_id: ExprId,
        package_id: PackageId,
        res: &Res,
        _input_params_map: &InputParamsMap,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        // TODO (cesarzc): Implement.
    }

    fn analyze_intrinsic_callable(
        callable: &CallableDecl,
        package_id: PackageId,
        package_store: &PackageStore,
    ) -> CallableComputeProps {
        assert!(Self::is_callable_intrinsic(callable));
        let package_pats = &package_store
            .0
            .get(package_id)
            .expect("Package should exist")
            .pats;
        match callable.kind {
            CallableKind::Function => Self::analyze_instrinsic_function(callable, package_pats),
            CallableKind::Operation => Self::analyze_instrinsic_operation(callable, package_pats),
        }
    }

    fn analyze_instrinsic_function(
        function: &CallableDecl,
        package_pats: &IndexMap<PatId, Pat>,
    ) -> CallableComputeProps {
        assert!(Self::is_callable_intrinsic(function));
        let input_params = NodeMap::from_callable_input(function, package_pats);
        // TODO (cesarzc): Set limit on the number of parameters.
        let mut apps = AppsTbl::new(input_params.0.len());
        for app_idx in 0..apps.max() {
            let input_params_app = InputParamsApp::from_app_idx(AppIdx(app_idx), &input_params);
            let app =
                Self::create_intrinsic_function_application(&input_params_app, &function.output);
            apps.push(app);
        }
        CallableComputeProps { apps }
    }

    fn analyze_instrinsic_operation(
        operation: &CallableDecl,
        package_pats: &IndexMap<PatId, Pat>,
    ) -> CallableComputeProps {
        assert!(Self::is_callable_intrinsic(operation));
        let input_params = NodeMap::from_callable_input(operation, package_pats);
        // TODO (cesarzc): Set limit on the number of parameters.
        let mut apps = AppsTbl::new(input_params.0.len());
        for app_idx in 0..apps.max() {
            let input_params_app = InputParamsApp::from_app_idx(AppIdx(app_idx), &input_params);
            let app =
                Self::create_intrinsic_operation_application(&input_params_app, &operation.output);
            apps.push(app);
        }
        CallableComputeProps { apps }
    }

    fn analyze_item(
        item: &Item,
        item_id: LocalItemId,
        package_id: PackageId,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        // TODO (cesarzc): use if let.
        //if let ItemKind::Callable(c) = item {
        //}

        //store_scratch.insert_item(package_id, item_id, ItemComputeProps::NonCallable);
        match &item.kind {
            ItemKind::Namespace(..) | ItemKind::Ty(..) => {
                store_scratch.insert_item(package_id, item_id, ItemComputeProps::NonCallable)
            }
            ItemKind::Callable(callable) => {
                Self::analyze_callable(callable, item_id, package_id, package_store, store_scratch)
            }
        };
    }

    fn analyze_non_intrinsic_callable(
        callable: &CallableDecl,
        callable_id: LocalItemId,
        package_id: PackageId,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        // Get the input params callable, and properly set the corresponding patterns.
        let package_pats = &package_store
            .0
            .get(package_id)
            .expect("`Package` should exist in `PackageStore`")
            .pats;
        let input_params_map = InputParamsMap::from_callable(callable, package_pats);

        // Initialize the callable applications table whose size depends on the number of input parameters.
        let mut callable_apps_tbl = AppsTbl::new(input_params_map.0.len());
        for _ in 0..callable_apps_tbl.max() {
            callable_apps_tbl.apps.push(ComputeProps {
                rt_caps: FxHashSet::default(),
                quantum_sources: Vec::new(),
            });
        }

        // Analyze each statement and update the callable apps table.
        let implementation_block_id = Self::get_callable_implementation_block_id(callable);
        let implementation_block = package_store
            .get_block(package_id, implementation_block_id)
            .expect("Block should exist");
        for stmt_id in &implementation_block.stmts {
            Self::analyze_stmt(
                *stmt_id,
                package_id,
                &input_params_map,
                package_store,
                store_scratch,
            );
            let _stmt_compute_props = store_scratch
                .get_stmt(&package_id, stmt_id)
                .expect("Statement was just analyzed");

            // TODO (cesarzc): need to expand the apps table based on this.
        }

        // TODO (cesarzc): analyze quantum sources based on the last statement.
        let callable_compute_props = CallableComputeProps {
            apps: callable_apps_tbl,
        };
        store_scratch.insert_item(
            package_id,
            callable_id,
            ItemComputeProps::Callable(callable_compute_props),
        );
    }

    fn analyze_stmt(
        stmt_id: StmtId,
        package_id: PackageId,
        input_params_map: &InputParamsMap,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        let stmt = package_store
            .get_stmt(package_id, stmt_id)
            .expect("Statement should exist");
        match stmt.kind {
            StmtKind::Expr(expr_id) | StmtKind::Semi(expr_id) => Self::analyze_stmt_expr(
                stmt_id,
                package_id,
                expr_id,
                input_params_map,
                package_store,
                store_scratch,
            ),
            StmtKind::Local(_, pat_id, expr_id) => Self::analyze_stmt_local(
                stmt_id,
                package_id,
                pat_id,
                expr_id,
                input_params_map,
                package_store,
                store_scratch,
            ),
            _ => {
                store_scratch.insert_stmt(package_id, stmt_id, ElmntComputeProps::Unsupported);
            }
        };
    }

    fn analyze_stmt_expr(
        stmt_id: StmtId,
        package_id: PackageId,
        expr_id: ExprId,
        input_params_map: &InputParamsMap,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        Self::analyze_expr(
            expr_id,
            package_id,
            input_params_map,
            package_store,
            store_scratch,
        );
        let expr_compute_props = store_scratch
            .get_expr(&package_id, &expr_id)
            .expect("Expression compute properties should exist since it has just been analyzed");

        let stmt_compute_props = ElmntComputeProps::from_expr_compute_props(expr_compute_props);
        store_scratch.insert_stmt(package_id, stmt_id, stmt_compute_props);
    }

    fn analyze_stmt_local(
        stmt_id: StmtId,
        package_id: PackageId,
        pat_id: PatId,
        expr_id: ExprId,
        input_params_map: &InputParamsMap,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        // Analyze the expression.
        Self::analyze_expr(
            expr_id,
            package_id,
            input_params_map,
            package_store,
            store_scratch,
        );

        // Propagate to patterns.
        Self::link_expr_to_local_pat(expr_id, pat_id, package_id, package_store, store_scratch);

        // Propagate to statement.
        Self::link_expr_to_stmt(expr_id, stmt_id, package_id, store_scratch);
    }

    fn create_intrinsic_function_application(
        input_params_app: &InputParamsApp,
        output_type: &Ty,
    ) -> ComputeProps {
        // Assume capabilities depending on which parameters are dynamic for the particular application.
        let mut rt_caps = FxHashSet::<RuntimeCapability>::default();
        for (node, param_compute_kind) in input_params_app.params_compute_kind.values() {
            if let NodeKind::InputParam(_, param_ty) = &node.kind {
                if let ComputeKind::Dynamic = param_compute_kind {
                    let param_caps = Foundational::assume_caps_from_type(param_ty);
                    rt_caps.extend(param_caps);
                }
            } else {
                panic!("Only input parameters are expected");
            }
        }

        // A function is purely classical if all its parameters are static.
        let is_purely_classical = input_params_app.idx.represents_all_static_params();

        // If this is not purely classical application then the output type affects the application capabilities.
        if !is_purely_classical {
            let output_caps = Foundational::assume_caps_from_type(output_type);
            rt_caps.extend(output_caps);
        }

        // If this is a purely classical application or the function output is `Unit` then this is not a quantum source.
        // Otherwise this is an intrinsic quantum source.
        let is_unit = matches!(*output_type, Ty::UNIT);
        let quantum_sources = if is_purely_classical || is_unit {
            Vec::new()
        } else {
            vec![QuantumSource::Intrinsic]
        };

        ComputeProps {
            rt_caps,
            quantum_sources,
        }
    }

    fn create_intrinsic_operation_application(
        input_params_app: &InputParamsApp,
        output_type: &Ty,
    ) -> ComputeProps {
        // Assume capabilities depending on which parameters are dynamic for the particular application.
        let mut rt_caps = FxHashSet::<RuntimeCapability>::default();
        for (node, param_compute_kind) in input_params_app.params_compute_kind.values() {
            if let NodeKind::InputParam(_, param_ty) = &node.kind {
                if let ComputeKind::Dynamic = param_compute_kind {
                    let param_caps = Foundational::assume_caps_from_type(param_ty);
                    rt_caps.extend(param_caps);
                }
            } else {
                panic!("Only input parameters are expected");
            }
        }

        // If this is not an all-static application then the output type affects the application capabilities.
        if input_params_app.idx.represents_all_static_params() {
            let output_caps = Foundational::assume_caps_from_type(output_type);
            rt_caps.extend(output_caps);
        }

        // If the operation is unit then it is an instrinsic quantum source.
        let is_unit = matches!(*output_type, Ty::UNIT);
        let quantum_sources = if is_unit {
            Vec::new()
        } else {
            vec![QuantumSource::Intrinsic]
        };

        ComputeProps {
            rt_caps,
            quantum_sources,
        }
    }

    fn get_callable_implementation_block_id(callable: &CallableDecl) -> BlockId {
        match callable.body.body {
            SpecBody::Impl(pat_id, block_id) => {
                if let Some(pid) = pat_id {
                    println!("{} | {}", callable.name, pid);
                }
                block_id
            }
            _ => panic!("Is not implementation"),
        }
    }

    fn is_callable_intrinsic(callable: &CallableDecl) -> bool {
        match callable.body.body {
            SpecBody::Gen(spec_gen) => spec_gen == SpecGen::Intrinsic,
            _ => false,
        }
    }

    fn link_expr_to_local_pat(
        expr_id: ExprId,
        pat_id: PatId,
        package_id: PackageId,
        package_store: &PackageStore,
        store_scratch: &mut StoreScratch,
    ) {
        // N.B. This function assumes the expression has already been analyzed.
        // TODO (cesarzc): The correct thing to do here would be to match the appropriate sub-expression to its corresponding node.
        let expr_compute_props = store_scratch
            .get_expr(&package_id, &expr_id)
            .expect("Expression compute properties must have already been analyzed.");
        let pat = package_store
            .get_pat(package_id, pat_id)
            .expect("Pattern must exist");
        match &pat.kind {
            PatKind::Bind(ident) => {
                let local_node = PatComputeProps::LocalNode(ident.id, expr_compute_props.clone());
                store_scratch.insert_pat(package_id, pat_id, local_node);
            }
            PatKind::Tuple(_) => {
                // TODO (cesarzc): Eventually we should be able to map each node to its corresponding sub-expression.
                store_scratch.insert_pat(package_id, pat_id, PatComputeProps::Unsupported)
            }
            PatKind::Discard => panic!("Discard patterns are not expected"),
        }
    }

    fn link_expr_to_stmt(
        expr_id: ExprId,
        stmt_id: StmtId,
        package_id: PackageId,
        store_scratch: &mut StoreScratch,
    ) {
        let expr_compute_props = store_scratch
            .get_expr(&package_id, &expr_id)
            .expect("Expression compute properties must have already been analyzed.");
        let stmt_compute_props = ElmntComputeProps::from_expr_compute_props(expr_compute_props);
        store_scratch.insert_stmt(package_id, stmt_id, stmt_compute_props);
    }
}

struct Foundational;

impl Foundational {
    pub fn assume_caps_from_type(ty: &Ty) -> FxHashSet<RuntimeCapability> {
        match ty {
            Ty::Array(_) => FxHashSet::from_iter([RuntimeCapability::HigherLevelConstructs]),
            Ty::Arrow(_) => FxHashSet::from_iter([RuntimeCapability::HigherLevelConstructs]),
            Ty::Prim(prim) => Self::assume_caps_from_primitive_type(prim),
            Ty::Tuple(v) => Self::assume_caps_from_tuple_type(v),
            Ty::Udt(_) => FxHashSet::from_iter([RuntimeCapability::HigherLevelConstructs]),
            _ => panic!("Unexpected type"),
        }
    }

    fn assume_caps_from_tuple_type(tuple: &[Ty]) -> FxHashSet<RuntimeCapability> {
        let mut caps = FxHashSet::<RuntimeCapability>::default();
        for item_type in tuple.iter() {
            let item_caps = Self::assume_caps_from_type(item_type);
            caps.extend(item_caps);
        }
        caps
    }

    fn assume_caps_from_primitive_type(primitive: &Prim) -> FxHashSet<RuntimeCapability> {
        match primitive {
            Prim::BigInt => FxHashSet::from_iter([RuntimeCapability::HigherLevelConstructs]),
            Prim::Bool => FxHashSet::from_iter([RuntimeCapability::ConditionalForwardBranching]),
            Prim::Double => FxHashSet::from_iter([RuntimeCapability::FloatingPointComputation]),
            Prim::Int => FxHashSet::from_iter([RuntimeCapability::IntegerComputations]),
            Prim::Pauli => FxHashSet::from_iter([RuntimeCapability::IntegerComputations]),
            Prim::Qubit => FxHashSet::default(),
            Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull => {
                FxHashSet::from_iter([RuntimeCapability::IntegerComputations])
            }
            Prim::Result => FxHashSet::from_iter([RuntimeCapability::ConditionalForwardBranching]),
            Prim::String => FxHashSet::from_iter([RuntimeCapability::HigherLevelConstructs]),
        }
    }
}

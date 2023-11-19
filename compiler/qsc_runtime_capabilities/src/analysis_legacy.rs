use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, CallableKind, ExprId, ItemId, ItemKind, LocalItemId, NodeId,
        Package, PackageId, PackageStore, Pat, PatId, PatKind, SpecBody, SpecGen, StmtId,
    },
    ty::{Prim, Ty},
};

use indenter::indented;
use rustc_hash::FxHashSet;
use std::{
    fmt::{self, Display, Formatter, Write},
    fs::File,
    io::Write as IoWrite,
};

use crate::{set_indentation, RuntimeCapability, StoreCapabilities};

// TODO (cesarzc): Use this throughout the code.
#[derive(Debug)]
pub struct AnalysisStore(IndexMap<PackageId, PackageAnalysis>);

impl Default for AnalysisStore {
    fn default() -> Self {
        AnalysisStore(IndexMap::new())
    }
}

// TODO (cesarzc): Rename this to PackageRuntimeProperties.
#[derive(Debug)]
struct PackageAnalysis {
    //pub callable_applications: IndexMap<LocalItemId, Option<Vec<CallableApplication>>>,
    pub callables: IndexMap<LocalItemId, Option<CallableAnalysis>>,
    pub blocks: IndexMap<BlockId, Option<BlockAnalysis>>,
    pub stmts: IndexMap<StmtId, Option<RuntimeProperties>>,
    pub exprs: IndexMap<ExprId, Option<RuntimeProperties>>,
    pub pats: IndexMap<PatId, Option<PatternAnalysis>>,
}

impl Display for PackageAnalysis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);

        // Display callables.
        write!(indent, "\ncallables:")?;
        for (id, callable) in self.callables.iter() {
            indent = set_indentation(indent, 1);
            write!(indent, "\nCallable: {id}")?;
            indent = set_indentation(indent, 2);
            if let Some(c) = callable {
                write!(indent, "{c}")?;
            } else {
                write!(indent, "\nNone")?;
            }
        }

        // Display blocks.
        indent = set_indentation(indent, 0);
        write!(indent, "\nblocks:")?;
        for (id, block) in self.blocks.iter() {
            indent = set_indentation(indent, 1);
            write!(indent, "\nBlock: {id}")?;
            indent = set_indentation(indent, 2);
            if let Some(b) = block {
                write!(indent, "{b}")?;
            } else {
                write!(indent, "\nNone")?;
            }
        }

        // Display statements.
        indent = set_indentation(indent, 0);
        write!(indent, "\nstatements:")?;
        for (id, statement) in self.stmts.iter() {
            indent = set_indentation(indent, 1);
            write!(indent, "\nStatement: {id}")?;
            indent = set_indentation(indent, 2);
            if let Some(s) = statement {
                write!(indent, "{s}")?;
            } else {
                write!(indent, "\nNone")?;
            }
        }

        // Display expressions.
        indent = set_indentation(indent, 0);
        write!(indent, "\nexpressions:")?;
        for (id, expression) in self.exprs.iter() {
            indent = set_indentation(indent, 1);
            write!(indent, "\nExpression: {id}")?;
            indent = set_indentation(indent, 2);
            if let Some(e) = expression {
                write!(indent, "{e}")?;
            } else {
                write!(indent, "\nNone")?;
            }
        }

        // Display patterns.
        indent = set_indentation(indent, 0);
        write!(indent, "\npatterns:")?;
        for (id, pattern) in self.pats.iter() {
            indent = set_indentation(indent, 1);
            write!(indent, "\nPattern: {id}")?;
            indent = set_indentation(indent, 2);
            if let Some(p) = pattern {
                write!(indent, "{p}")?;
            } else {
                write!(indent, "\nNone")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableApplicationIndex(usize);

pub struct CallableApplicationId {
    pub callable_id: ItemId,
    pub application_index: CallableApplicationIndex,
}

#[derive(Debug)]
struct CallableApplication {
    // CONSIDER (cesarzc): Might be helpful to have the (global) ItemId and the application index?
    pub is_quantum_source: bool,
    pub runtime_capabilities: FxHashSet<RuntimeCapability>,
    // CONSIDER (cesarzc): Maybe we can add diagnostic information here such as expression ID to provide good errors to
    // the user when checking against a target.
}

// CONSIDER (cesarzc): Might need to do this a per specialization basis.
#[derive(Debug)]
struct CallableAnalysis {
    pub inherent_properties: Option<RuntimeProperties>,
    // CONSIDER (cesarzc): It might make sense to set this to pattern IDs that match to the resolutions to avoid data duplication.
    pub params_properties: Option<Vec<RuntimeProperties>>,
}

impl Display for CallableAnalysis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        let inherent_caps = match &self.inherent_properties {
            None => "None".to_string(),
            Some(caps) => format!("{caps}"),
        };
        write!(indent, "\ninherent_caps: {inherent_caps}")?;
        write!(indent, "\nparameter_caps:")?;
        if let Some(param_caps) = &self.params_properties {
            indent = set_indentation(indent, 1);
            for cap in param_caps {
                write!(indent, "{cap}")?;
            }
        } else {
            write!(f, " NONES")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum PatternAnalysis {
    IntrinsicCallableParameter,
    CallableParameterTuple,
    CallableParameterIdent(Vec<RuntimeCapability>),
    Ident(RuntimeProperties),
}

impl Display for PatternAnalysis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            PatternAnalysis::IntrinsicCallableParameter => {
                write!(indent, "\nIntrinsicCallableParameter")?
            }
            PatternAnalysis::CallableParameterTuple => write!(indent, "\nCallableParameterTuple")?,
            PatternAnalysis::CallableParameterIdent(caps) => {
                write!(indent, "\nCallableParameterIdent: {{")?;
                indent = set_indentation(indent, 1);
                for cap in caps {
                    write!(indent, "\n{cap:?}")?;
                }
                indent = set_indentation(indent, 0);
                write!(indent, "\n}}")?;
            }
            PatternAnalysis::Ident(properties) => {
                write!(indent, "\nIdent")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{properties}")?;
            }
        };
        Ok(())
    }
}

// CONSIDER (cesarzc): This might change a bit since not all blocks are callable blocks.
#[derive(Debug)]
struct BlockAnalysis {
    pub inherent_properties: Option<RuntimeProperties>,
    pub params_properties: Option<Vec<RuntimeProperties>>,
}

impl Display for BlockAnalysis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        let inherent_caps = match &self.inherent_properties {
            None => "None".to_string(),
            Some(caps) => format!("{caps}"),
        };
        write!(indent, "\ninherent_caps: {inherent_caps}")?;
        write!(indent, "\nparameter_caps:")?;
        if let Some(param_caps) = &self.params_properties {
            indent = set_indentation(indent, 1);
            for cap in param_caps {
                write!(indent, "{cap}")?;
            }
        } else {
            write!(f, " NONES")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RuntimeProperties {
    pub is_quantum_source: Option<bool>,
    pub caps: Option<FxHashSet<RuntimeCapability>>,
}

impl Display for RuntimeProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 1);
        let is_quantum_source = match self.is_quantum_source {
            None => "None".to_string(),
            Some(iqs) => format!("{iqs}"),
        };
        write!(indent, "\nis_quantum_source: {}", is_quantum_source)?;
        write!(indent, "\ncapabilities:")?;
        if let Some(caps) = &self.caps {
            write!(indent, "\n{{")?;
            indent = set_indentation(indent, 2);
            for capability in caps.iter() {
                write!(indent, "\n{capability:?}")?;
            }
            indent = set_indentation(indent, 1);
            write!(indent, "\n}}")?;
        } else {
            write!(f, "None")?;
        }
        Ok(())
    }
}

// DBG (cesarzc): For debugging purposes only.
fn save_package_analysis_to_files(store: &AnalysisStore, phase: u8) {
    for (id, package) in store.0.iter() {
        let filename = format!("dbg/phase{phase}.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}

pub struct LegacyAnalyzer<'a> {
    package_store: &'a PackageStore,
    analysis_store: AnalysisStore,
}

impl<'a> LegacyAnalyzer<'a> {
    pub fn new(package_store: &'a PackageStore) -> Self {
        let mut initializer = Initializer::new(package_store);
        let analysis_store = initializer.create_analysis_store();
        Self {
            package_store,
            analysis_store,
        }
    }

    pub fn run(&mut self) -> StoreCapabilities {
        save_package_analysis_to_files(&self.analysis_store, 0);
        let mut functions_analyzer =
            FunctionsAnalyzer::new(self.package_store, &mut self.analysis_store);
        functions_analyzer.run();
        save_package_analysis_to_files(&self.analysis_store, 1);
        // TODO: Complete implementation.
        StoreCapabilities(IndexMap::new())
    }
}

struct Initializer<'a> {
    package_store: &'a PackageStore,
}

impl<'a> Initializer<'a> {
    pub fn new(package_store: &'a PackageStore) -> Self {
        Self { package_store }
    }

    pub fn create_analysis_store(&mut self) -> AnalysisStore {
        let mut analysis_store = AnalysisStore::default();
        for (id, package) in self.package_store.0.iter() {
            let package_analysis = self.create_package_analysis(package);
            analysis_store.0.insert(id, package_analysis);
        }
        analysis_store
    }

    fn create_package_analysis(&mut self, package: &Package) -> PackageAnalysis {
        // Initialize blocks.
        let mut blocks = IndexMap::<BlockId, Option<BlockAnalysis>>::new();
        for (id, _) in package.blocks.iter() {
            blocks.insert(id, None);
        }

        // Initialize statements.
        let mut stmts = IndexMap::<StmtId, Option<RuntimeProperties>>::new();
        for (id, _) in package.stmts.iter() {
            stmts.insert(id, None);
        }

        // Initialize expressions.
        let mut exprs = IndexMap::<ExprId, Option<RuntimeProperties>>::new();
        for (id, _) in package.exprs.iter() {
            exprs.insert(id, None);
        }

        // Initialize patterns.
        let mut pats = IndexMap::<PatId, Option<PatternAnalysis>>::new();
        for (id, _) in package.pats.iter() {
            pats.insert(id, None);
        }

        // Initialize callables.
        // COSIDER (cesarzc): Might be important for callables to be done last.
        let mut callables = IndexMap::<LocalItemId, Option<CallableAnalysis>>::new();
        for (id, item) in package.items.iter() {
            let capabilities = match &item.kind {
                ItemKind::Callable(c) => {
                    Some(self.create_callable_analysis(c, &package.pats, &mut pats))
                }
                _ => None,
            };
            callables.insert(id, capabilities);
        }

        PackageAnalysis {
            callables,
            blocks,
            stmts,
            exprs,
            pats,
        }
    }

    fn create_callable_analysis(
        &mut self,
        callable: &CallableDecl,
        store_patterns: &IndexMap<PatId, Pat>, // N.B. Needed for figuring out initial parameter analysis.
        analysis_patterns: &mut IndexMap<PatId, Option<PatternAnalysis>>, // N.B. Needed for storing initial parameter analysis.
    ) -> CallableAnalysis {
        match callable.kind {
            CallableKind::Function => {
                self.create_function_analysis(callable, store_patterns, analysis_patterns)
            }
            CallableKind::Operation => {
                self.create_operation_analysis(callable, store_patterns, analysis_patterns)
            }
        }
    }

    fn create_function_analysis(
        &mut self,
        function: &CallableDecl,
        store_patterns: &IndexMap<PatId, Pat>,
        analysis_patterns: &mut IndexMap<PatId, Option<PatternAnalysis>>,
    ) -> CallableAnalysis {
        // CONSIDER (cesarzc): Maybe only set inherent properties for functions at this stage.
        // Inherent properties for all functions are the same.
        let inherent_properties = Some(RuntimeProperties {
            is_quantum_source: Some(false),
            caps: Some(FxHashSet::default()),
        });

        // At this stage, parameters' properties can only be determined for `body instrinsic`
        // functions.
        let mut params_properties = None;
        let is_intrinsic = is_intrinsic(function);
        if is_intrinsic {
            // Determine whether parameters can be affect whether the function becomes a quantum
            // source.
            let is_unit = matches!(function.output, Ty::UNIT);
            let is_quantum_source = !is_unit;

            // Determine the parameters' properties based on the input pattern.
            let input_pattern = store_patterns
                .get(function.input)
                .expect("Pattern should exist.");

            let input_params_types = get_params_types_from_pattern(input_pattern);
            let mut runtime_properties = Vec::<RuntimeProperties>::new();
            for param_type in input_params_types {
                let mut param_caps = FxHashSet::<RuntimeCapability>::default();
                let caps = get_capabilities_for_type(&param_type);
                caps.iter().for_each(|c| _ = param_caps.insert(c.clone()));
                let properties = RuntimeProperties {
                    is_quantum_source: Some(is_quantum_source),
                    caps: Some(param_caps),
                };
                runtime_properties.push(properties);
            }
            params_properties = Some(runtime_properties);

            // Now that the runtime properties of the function's parameters has been determined,
            // mark the patterns related to the input parameters appropriately.
            self.set_input_params_pattern_analysis(input_pattern, analysis_patterns);
        }

        CallableAnalysis {
            inherent_properties,
            params_properties,
        }
    }

    fn create_operation_analysis(
        &mut self,
        operation: &CallableDecl,
        store_patterns: &IndexMap<PatId, Pat>,
        analysis_patterns: &mut IndexMap<PatId, Option<PatternAnalysis>>,
    ) -> CallableAnalysis {
        let is_intrinsic = is_intrinsic(operation);

        // Analysis `body intrinsic` operations is complete at initialization.
        if is_intrinsic {
            // Determine whether the operation is an inherent quantum source.
            let is_unit = matches!(operation.output, Ty::UNIT);
            let is_quantum_source = !is_unit;

            // Intrinsic operations have no inherent capabilities.
            let inherent_caps = FxHashSet::default();
            let inherent_properties = RuntimeProperties {
                is_quantum_source: Some(is_quantum_source),
                caps: Some(inherent_caps),
            };

            // Determine the parameters' properties based on the input pattern.
            let input_pattern = store_patterns
                .get(operation.input)
                .expect("Pattern should exist.");

            let input_params_types = get_params_types_from_pattern(input_pattern);
            let mut params_properties = Vec::<RuntimeProperties>::new();
            for param_type in input_params_types {
                let mut param_caps = FxHashSet::<RuntimeCapability>::default();
                let caps = get_capabilities_for_type(&param_type);
                caps.iter().for_each(|c| _ = param_caps.insert(c.clone()));
                let properties = RuntimeProperties {
                    // The `is_quantum_source` property for all parameters is the
                    is_quantum_source: Some(is_quantum_source),
                    caps: Some(param_caps),
                };
                params_properties.push(properties);
            }

            // Now that the runtime properties of the operation's parameters has been determined,
            // mark the patterns related to the input parameters appropriately.
            self.set_input_params_pattern_analysis(input_pattern, analysis_patterns);

            // Return the created callable analysis object.
            return CallableAnalysis {
                inherent_properties: Some(inherent_properties),
                params_properties: Some(params_properties),
            };
        }

        // Analysis for operations that are not `body intrinsic` will be performed at later phases.
        CallableAnalysis {
            inherent_properties: None,
            params_properties: None,
        }
    }

    fn set_input_params_pattern_analysis(
        &mut self,
        pattern: &Pat,
        analysis_patterns: &mut IndexMap<PatId, Option<PatternAnalysis>>,
    ) {
        analysis_patterns.insert(
            pattern.id,
            Some(PatternAnalysis::IntrinsicCallableParameter),
        );
        match &pattern.kind {
            PatKind::Bind(_) => {} // Nothing else left to do.
            PatKind::Tuple(ident_patterns) => {
                for pat_id in ident_patterns.iter() {
                    analysis_patterns
                        .insert(*pat_id, Some(PatternAnalysis::IntrinsicCallableParameter));
                }
            }
            _ => panic!("Only callable parameter patterns are expected"),
        }
    }
}

struct FunctionsAnalyzer<'a, 'b> {
    package_store: &'a PackageStore,
    analysis_store: &'b mut AnalysisStore,
}

impl<'a, 'b> FunctionsAnalyzer<'a, 'b> {
    pub fn new(package_store: &'a PackageStore, analysis_store: &'b mut AnalysisStore) -> Self {
        Self {
            package_store,
            analysis_store,
        }
    }

    pub fn run(&mut self) {
        for (id, package) in self.package_store.0.iter() {
            let mut package_analysis = self
                .analysis_store
                .0
                .get_mut(id)
                .expect("`PackageAnalysis` should exist");
            Self::analyze_package_functions(package, package_analysis);
        }
    }

    fn analyze_package_functions(package: &Package, package_analysis: &mut PackageAnalysis) {
        for (item_id, item) in package.items.iter() {
            // Only analyze functions.
            if let ItemKind::Callable(callable) = &item.kind {
                if callable.kind == CallableKind::Function {
                    Self::analyze_function(item_id, callable, package, package_analysis);
                }
            }
        }
    }

    fn analyze_function(
        function_id: LocalItemId,
        function: &CallableDecl,
        package: &Package,
        package_analysis: &mut PackageAnalysis,
    ) {
        let function_analysis = package_analysis
            .callables
            .get_mut(function_id)
            .expect("`CallableAnalysis` should exist")
            .as_mut()
            .expect("`CallableAnalysis` should be initialized");

        // Do the analysis if the parameters' properties have not been set.
        if function_analysis.params_properties.is_none() {
            let store_patterns = &package.pats;
            let input_pattern = store_patterns
                .get(function.input)
                .expect("Pattern should exist.");
            let params_resolutions = get_params_resolutions(input_pattern, store_patterns);
            // CONSIDER (cesarzc): This probably has to be done for all specializations, not just body.
            let function_block = match function.body.body {
                SpecBody::Impl(_, block_id) => {
                    package.blocks.get(block_id).expect("`Block` should exist")
                }
                _ => panic!("Automatically generated specializations are not supported."),
            };

            // Determine the properties for each parameter.
            let mut params_properties = Vec::<RuntimeProperties>::new();
            for resolution in params_resolutions {
                let caps = Self::get_capabilities_for_resolution(resolution, function_block);
                let properties = RuntimeProperties {
                    is_quantum_source: Some(true), // TODO (cesarzc): Do this properly instead of using shortcut.
                    caps: Some(caps),
                };
                params_properties.push(properties);
            }

            function_analysis.params_properties = Some(params_properties);
        }
    }

    fn get_capabilities_for_resolution(
        resolution: NodeId,
        block: &Block,
    ) -> FxHashSet<RuntimeCapability> {
        let mut caps = FxHashSet::<RuntimeCapability>::default();
        // TODO (cesarzc): Implement.
        caps
    }
}

fn get_capabilities_for_types(tuple: &[Ty]) -> Vec<RuntimeCapability> {
    let mut caps = Vec::<RuntimeCapability>::default();
    for item_type in tuple.iter() {
        let item_caps = get_capabilities_for_type(item_type);
        caps.extend(item_caps);
    }
    caps
}

fn get_capabilities_for_type(ty: &Ty) -> Vec<RuntimeCapability> {
    match ty {
        Ty::Array(_) => vec![RuntimeCapability::HigherLevelConstructs],
        Ty::Arrow(_) => vec![RuntimeCapability::HigherLevelConstructs],
        Ty::Prim(prim) => get_capabilities_for_primitive_type(prim),
        Ty::Tuple(v) => get_capabilities_for_types(v),
        Ty::Udt(_) => vec![RuntimeCapability::HigherLevelConstructs],
        _ => panic!("Unexpected type"),
    }
}

fn get_capabilities_for_primitive_type(primitive: &Prim) -> Vec<RuntimeCapability> {
    match primitive {
        Prim::BigInt => vec![RuntimeCapability::HigherLevelConstructs],
        Prim::Bool => vec![RuntimeCapability::ConditionalForwardBranching],
        Prim::Double => vec![RuntimeCapability::FloatingPointComputation],
        Prim::Int => vec![RuntimeCapability::IntegerComputations],
        Prim::Pauli => vec![RuntimeCapability::IntegerComputations],
        Prim::Qubit => vec![],
        Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull => {
            vec![RuntimeCapability::IntegerComputations]
        }
        Prim::Result => vec![RuntimeCapability::ConditionalForwardBranching],
        Prim::String => vec![RuntimeCapability::HigherLevelConstructs],
    }
}

fn get_params_resolutions(pattern: &Pat, store_patterns: &IndexMap<PatId, Pat>) -> Vec<NodeId> {
    let mut resolutions = Vec::<NodeId>::new();
    match &pattern.kind {
        PatKind::Bind(ident) => resolutions.push(ident.id),
        PatKind::Tuple(pattern_ids) => {
            for pat_id in pattern_ids {
                let pat = store_patterns.get(*pat_id).expect("`Pattern` should exist");
                // CONSIDER (cesarzc): Goes into nested params, consider it when using data produced by this function.
                let mut param_resolutions = get_params_resolutions(pat, store_patterns);
                resolutions.append(&mut param_resolutions);
            }
        }
        _ => panic!("Only callable parameter patterns are expected"),
    };
    resolutions
}

fn get_params_types_from_pattern(pattern: &Pat) -> Vec<Ty> {
    match pattern.kind {
        PatKind::Bind(_) => match pattern.ty {
            Ty::Array(_) | Ty::Arrow(_) | Ty::Prim(_) | Ty::Tuple(_) | Ty::Udt(_) => {
                vec![pattern.ty.clone()]
            }
            _ => panic!(
                "Unexpected pattern type {} for pattern {}",
                pattern.ty, pattern.id
            ),
        },
        PatKind::Tuple(_) => match &pattern.ty {
            Ty::Tuple(vector) => vector.clone(),
            _ => panic!("Unexpected pattern type"),
        },
        _ => panic!("Only callable parameter patterns are expected"),
    }
}

fn is_intrinsic(callable: &CallableDecl) -> bool {
    match callable.body.body {
        SpecBody::Gen(spec_gen) => spec_gen == SpecGen::Intrinsic,
        _ => false,
    }
}

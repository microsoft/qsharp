use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        BlockId, CallableDecl, CallableKind, ExprId, ItemKind, LocalItemId, Package, PackageId,
        PackageStore, Pat, PatId, PatKind, SpecBody, SpecGen, StmtId,
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
    pub callables: IndexMap<LocalItemId, Option<CallableAnalysis>>,
    pub blocks: IndexMap<BlockId, Option<BlockAnalysis>>,
    pub stmts: IndexMap<StmtId, Option<RuntimePropeties>>,
    pub exprs: IndexMap<ExprId, Option<RuntimePropeties>>,
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

// CONSIDER (cesarzc): Might need to do this a per specialization basis.
#[derive(Debug)]
struct CallableAnalysis {
    pub inherent_properties: Option<RuntimePropeties>,
    pub params_properties: Option<Vec<RuntimePropeties>>,
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
            write!(f, " None")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum PatternAnalysis {
    IntrinsicCallableParameter,
    CallableParameterTuple,
    CallableParameterIdent(Vec<RuntimeCapability>),
    Ident(RuntimePropeties),
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
    pub inherent_properties: Option<RuntimePropeties>,
    pub params_properties: Option<Vec<RuntimePropeties>>,
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
            write!(f, "None")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RuntimePropeties {
    pub is_quantum_source: Option<bool>,
    pub caps: Option<FxHashSet<RuntimeCapability>>,
}

impl Display for RuntimePropeties {
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

pub struct Analyzer<'a> {
    package_store: &'a PackageStore,
    analysis_store: AnalysisStore,
}

impl<'a> Analyzer<'a> {
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
        let mut stmts = IndexMap::<StmtId, Option<RuntimePropeties>>::new();
        for (id, _) in package.stmts.iter() {
            stmts.insert(id, None);
        }

        // Initialize expressions.
        let mut exprs = IndexMap::<ExprId, Option<RuntimePropeties>>::new();
        for (id, _) in package.exprs.iter() {
            exprs.insert(id, None);
        }

        // Initialize patterns.
        let mut pats = IndexMap::<PatId, Option<PatternAnalysis>>::new();
        for (id, _) in package.pats.iter() {
            pats.insert(id, None);
        }

        // Initialize callables.
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
        _callable: &CallableDecl,
        store_patterns: &IndexMap<PatId, Pat>,
        analysis_patterns: &mut IndexMap<PatId, Option<PatternAnalysis>>,
    ) -> CallableAnalysis {
        let inherent_caps = Some(RuntimePropeties {
            is_quantum_source: Some(false),
            caps: Some(FxHashSet::default()),
        });

        CallableAnalysis {
            inherent_properties: inherent_caps,
            params_properties: None, // TODO (cesarzc): Populare correctly.
        }
    }

    fn create_operation_analysis(
        &mut self,
        operation: &CallableDecl,
        store_patterns: &IndexMap<PatId, Pat>,
        analysis_patterns: &mut IndexMap<PatId, Option<PatternAnalysis>>,
    ) -> CallableAnalysis {
        let is_intrinsic = Self::is_intrinsic(operation);

        // Analysis `body intrinsic` operations is complete at initialization.
        if is_intrinsic {
            // Determine whether the operation is an inherent quantum source.
            let is_unit = matches!(operation.output, Ty::UNIT);
            let is_quantum_source = is_intrinsic && !is_unit;

            // Intrinsic operations have no inherent capabilities.
            let inherent_caps = FxHashSet::default();
            let inherent_properties = RuntimePropeties {
                is_quantum_source: Some(is_quantum_source),
                caps: Some(inherent_caps),
            };

            // Determine the parameters' properties based on the input pattern.
            let input_pattern = store_patterns
                .get(operation.input)
                .expect("Pattern should exist.");

            let input_params_types = Self::get_params_types_from_pattern(input_pattern);
            let mut params_properties = Vec::<RuntimePropeties>::new();
            for param_type in input_params_types {
                let mut param_caps = FxHashSet::<RuntimeCapability>::default();
                let caps = get_capabilities_for_type(&param_type);
                for cap in caps {
                    param_caps.insert(cap);
                }
                let properties = RuntimePropeties {
                    is_quantum_source: Some(is_quantum_source), // Parameters share the same
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

    // CONSIDER (cesarzc): Might get reused in some other place.
    fn get_params_types_from_pattern(pattern: &Pat) -> Vec<Ty> {
        match pattern.kind {
            PatKind::Bind(_) => match pattern.ty {
                Ty::Array(_) | Ty::Arrow(_) | Ty::Prim(_) | Ty::Tuple(_) | Ty::Udt(_) => {
                    vec![pattern.ty.clone()]
                }
                _ => panic!("Unexpected pattern type"),
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

    pub fn run(&mut self) {}
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
        Prim::Double => vec![RuntimeCapability::FloatingPointComputationg],
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

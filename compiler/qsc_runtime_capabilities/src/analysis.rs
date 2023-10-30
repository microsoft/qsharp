use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        BlockId, CallableDecl, ExprId, ItemKind, LocalItemId, Package, PackageId, PackageStore,
        PatId, SpecBody, SpecGen, StmtId,
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
    pub blocks: IndexMap<BlockId, Option<BlockCapsAnalysis>>,
    pub stmts: IndexMap<StmtId, Option<RuntimePropeties>>,
    pub exprs: IndexMap<ExprId, Option<RuntimePropeties>>,
    pub pats: IndexMap<PatId, Option<RuntimePropeties>>,
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
    pub inherent_caps: Option<RuntimePropeties>,
    pub parameter_caps: Option<Vec<RuntimePropeties>>,
}

impl Display for CallableAnalysis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        let inherent_caps = match &self.inherent_caps {
            None => "None".to_string(),
            Some(caps) => format!("{caps}"),
        };
        write!(indent, "\ninherent_caps: {inherent_caps}")?;
        write!(indent, "\nparameter_caps:")?;
        if let Some(param_caps) = &self.parameter_caps {
            indent = set_indentation(indent, 1);
            for cap in param_caps {
                write!(indent, "\n{cap}")?;
            }
        } else {
            write!(f, " None")?;
        }
        Ok(())
    }
}

// CONSIDER (cesarzc): This seems the same as `CallableCapsScaffolding`.
#[derive(Debug)]
struct BlockCapsAnalysis {
    pub inherent_caps: Option<RuntimePropeties>,
    pub parameter_caps: Option<Vec<RuntimePropeties>>,
}

impl Display for BlockCapsAnalysis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        let inherent_caps = match &self.inherent_caps {
            None => "None".to_string(),
            Some(caps) => format!("{caps}"),
        };
        write!(indent, "\ninherent_caps: {inherent_caps}")?;
        write!(indent, "\nparameter_caps:")?;
        if let Some(param_caps) = &self.parameter_caps {
            indent = set_indentation(indent, 1);
            for cap in param_caps {
                write!(indent, "\n{cap}")?;
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
        let mut indent = set_indentation(indented(f), 0);
        let is_quantum_source = match self.is_quantum_source {
            None => "None".to_string(),
            Some(iqs) => format!("{iqs}"),
        };
        write!(indent, "\nis_quantum_source: {}", is_quantum_source)?;
        write!(indent, "\ncapabilities:")?;
        if let Some(caps) = &self.caps {
            indent = set_indentation(indent, 1);
            for capability in caps.iter() {
                write!(indent, "\n{capability:?}")?;
            }
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
        // TODO: Do something.
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
            let package_analysis = self.from_package(package);
            analysis_store.0.insert(id, package_analysis);
        }
        analysis_store
    }

    fn from_package(&mut self, package: &Package) -> PackageAnalysis {
        // Initialize callables.
        let mut callables = IndexMap::<LocalItemId, Option<CallableAnalysis>>::new();
        for (id, item) in package.items.iter() {
            let capabilities = match &item.kind {
                ItemKind::Callable(c) => Some(Self::from_callable(c)),
                _ => None,
            };
            callables.insert(id, capabilities);
        }

        // Initialize blocks.
        let mut blocks = IndexMap::<BlockId, Option<BlockCapsAnalysis>>::new();
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
        let mut pats = IndexMap::<PatId, Option<RuntimePropeties>>::new();
        for (id, _) in package.pats.iter() {
            pats.insert(id, None);
        }

        PackageAnalysis {
            callables,
            blocks,
            stmts,
            exprs,
            pats,
        }
    }

    fn from_callable(callable: &CallableDecl) -> CallableAnalysis {
        // TODO (cesarzc): Separate into from_function and from_operation.

        // Parameter capabilities for QIS callables depend on the parameter type.
        // E.g.: Int -> {IntegerComputations}, Double -> {FloatingPointComputations}, Qubit -> {}.
        let is_qis_callable = callable.name.name.starts_with("__quantum__qis");
        // TODO (cesarzc): Implement.
        let parameter_caps = None;

        //
        let is_output_type_result = match callable.output {
            Ty::Prim(p) => p == Prim::Result,
            _ => false,
        };
        let is_quantum_source = is_output_type_result && is_qis_callable;
        let mut intrinsic_caps = None;
        if is_quantum_source {
            intrinsic_caps = Some(RuntimePropeties {
                is_quantum_source: Some(true),
                caps: Some(FxHashSet::default()),
            });
        }

        CallableAnalysis {
            inherent_caps: intrinsic_caps,
            parameter_caps,
        }
    }

    fn from_function(callable: &CallableDecl) -> CallableAnalysis {
        let inherent_caps = Some(RuntimePropeties {
            is_quantum_source: Some(false),
            caps: Some(FxHashSet::default()),
        });

        CallableAnalysis {
            inherent_caps,
            parameter_caps: None,
        }
    }

    fn is_intrinsic(callable: &CallableDecl) -> bool {
        match callable.body.body {
            SpecBody::Gen(spec_gen) => spec_gen == SpecGen::Intrinsic,
            _ => false,
        }
    }
}

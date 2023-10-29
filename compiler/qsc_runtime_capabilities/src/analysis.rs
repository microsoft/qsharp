use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        BlockId, CallableDecl, ExprId, ItemKind, LocalItemId, Package, PackageId, PackageStore,
        PatId, StmtId,
    },
    ty::{Prim, Ty},
};

use indenter::indented;
use std::{
    fmt::{self, Display, Formatter, Write},
    fs::File,
    io::Write as IoWrite,
};

use crate::{set_indentation, RuntimePropeties, StoreCapabilities};

#[derive(Debug)]
struct PackageCapsScaffolding {
    pub callables: IndexMap<LocalItemId, Option<CallableCapsScaffolding>>,
    pub blocks: IndexMap<BlockId, Option<BlockCapsScaffolding>>,
    pub stmts: IndexMap<StmtId, Option<RuntimePropeties>>,
    pub exprs: IndexMap<ExprId, Option<RuntimePropeties>>,
    pub pats: IndexMap<PatId, Option<RuntimePropeties>>,
}

impl Display for PackageCapsScaffolding {
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
struct CallableCapsScaffolding {
    pub intrinsic_caps: Option<RuntimePropeties>,
    pub parameter_caps: Option<Vec<RuntimePropeties>>,
}

impl Display for CallableCapsScaffolding {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "\nis_quantum_source: {:?}", self.intrinsic_caps)?;
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
struct BlockCapsScaffolding {
    pub intrinsic_caps: Option<RuntimePropeties>,
    pub parameter_caps: Option<Vec<RuntimePropeties>>,
}

impl Display for BlockCapsScaffolding {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "\nis_quantum_source: {:?}", self.intrinsic_caps)?;
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

// DBG (cesarzc): For debugging purposes only.
fn save_package_scaffoldings_to_files(
    store: &IndexMap<PackageId, PackageCapsScaffolding>,
    phase: u8,
) {
    for (id, package) in store.iter() {
        let filename = format!("dbg/phase{phase}.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}

pub struct Analyzer {
    store: IndexMap<PackageId, PackageCapsScaffolding>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            store: IndexMap::new(),
        }
    }

    pub fn analyze_runtime_capabilities(&mut self, store: &PackageStore) -> StoreCapabilities {
        self.initialize(store);
        save_package_scaffoldings_to_files(&self.store, 0);
        // TODO (cesarzc): should convert the store somehow.
        StoreCapabilities(IndexMap::new())
    }

    fn initialize(&mut self, store: &PackageStore) {
        for (id, package) in store.0.iter() {
            let capabilities = Initializer::from_package(package);
            self.store.insert(id, capabilities);
        }
    }
}

struct Initializer;

impl Initializer {
    pub fn from_package(package: &Package) -> PackageCapsScaffolding {
        // Initialize callables.
        let mut callables = IndexMap::<LocalItemId, Option<CallableCapsScaffolding>>::new();
        for (id, item) in package.items.iter() {
            let capabilities = match &item.kind {
                ItemKind::Callable(c) => Some(Self::from_callable(c)),
                _ => None,
            };
            callables.insert(id, capabilities);
        }

        // Initialize blocks.
        let mut blocks = IndexMap::<BlockId, Option<BlockCapsScaffolding>>::new();
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

        PackageCapsScaffolding {
            callables,
            blocks,
            stmts,
            exprs,
            pats,
        }
    }

    fn from_callable(callable: &CallableDecl) -> CallableCapsScaffolding {
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
                is_quantum_source: true,
                caps: Vec::new(),
            });
        }

        CallableCapsScaffolding {
            intrinsic_caps,
            parameter_caps,
        }
    }
}

use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{BlockId, ExprId, LocalItemId, PackageId};

use indenter::{indented, Indented};
use std::fmt::{self, Display, Formatter, Write};

pub mod analysis;

fn set_indentation<'a, 'b>(
    indent: Indented<'a, Formatter<'b>>,
    level: usize,
) -> Indented<'a, Formatter<'b>> {
    match level {
        0 => indent.with_str(""),
        1 => indent.with_str("    "),
        2 => indent.with_str("        "),
        3 => indent.with_str("            "),
        _ => unimplemented!("intentation level not supported"),
    }
}

#[derive(Debug)]
pub enum RuntimeCapability {
    ConditionalForwardBranching,
    QubitReuse,
    IntegerComputations,
    FloatingPointComputationg,
    BackwardsBranching,
    UserDefinedFunctionCalls,
    HigherLevelConstructs,
}

#[derive(Debug)]
pub struct Capabilities(Vec<RuntimeCapability>);

impl Display for Capabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        for capability in self.0.iter() {
            write!(indent, "\n{capability:?}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CallableCapabilities {
    pub is_quantum_source: bool,
    pub inherent: Capabilities,
}

impl Display for CallableCapabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "\nis_quantum_source: {}", self.is_quantum_source)?;
        write!(indent, "\ninherent: {}", self.inherent)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BlockCapabilities {
    pub inherent: Vec<RuntimeCapability>,
}

impl Display for BlockCapabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "\ninherent:")?;
        indent = set_indentation(indent, 1);
        for capability in self.inherent.iter() {
            write!(indent, "\n{capability:?}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct StatementCapabilities {
    pub inherent: Option<Vec<RuntimeCapability>>,
}

impl Display for StatementCapabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "\ninherent:")?;
        match self.inherent {
            None => write!(indent, "None")?,
            Some(_) => write!(indent, "Some")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExpressionCapabilities {
    pub inherent: Vec<RuntimeCapability>,
}

impl Display for ExpressionCapabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "\ninherent:")?;
        indent = set_indentation(indent, 1);
        for capability in self.inherent.iter() {
            write!(indent, "\n{capability:?}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct PackageCapabilities {
    pub callables: IndexMap<LocalItemId, Option<CallableCapabilities>>,
    pub blocks: IndexMap<BlockId, BlockCapabilities>,
    pub expressions: IndexMap<ExprId, ExpressionCapabilities>,
}

impl Display for PackageCapabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Package:")?;

        // Display callables.
        indent = set_indentation(indent, 1);
        write!(indent, "\ncallables:")?;
        for (id, capabilities) in self.callables.iter() {
            indent = set_indentation(indent, 2);
            write!(indent, "\nCallable: {id}")?;
            indent = set_indentation(indent, 3);
            match capabilities {
                None => write!(indent, "\nNone")?,
                Some(c) => write!(indent, "{c}")?,
            }
        }

        // Display blocks.
        indent = set_indentation(indent, 1);
        write!(indent, "\nblocks:")?;
        for (id, block) in self.blocks.iter() {
            indent = set_indentation(indent, 2);
            write!(indent, "\nBlock: {id}")?;
            indent = set_indentation(indent, 3);
            write!(indent, "{block}")?;
        }

        // Display expressions.
        indent = set_indentation(indent, 1);
        write!(indent, "\nexpressions:")?;
        for (id, expression) in self.expressions.iter() {
            indent = set_indentation(indent, 2);
            write!(indent, "\nExpression: {id}")?;
            indent = set_indentation(indent, 3);
            write!(indent, "{expression}")?;
        }

        Ok(())
    }
}

pub struct StoreCapabilities(pub IndexMap<PackageId, PackageCapabilities>);

impl Display for StoreCapabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        for (id, package_capabilities) in self.0.iter() {
            write!(indent, "\n|{id}|\n{package_capabilities}")?;
        }
        Ok(())
    }
}

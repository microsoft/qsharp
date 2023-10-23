use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{LocalItemId, PackageId};

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
pub struct CallableCapabilities {
    pub inherent: Vec<RuntimeCapability>,
}

impl Default for CallableCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

impl CallableCapabilities {
    pub fn new() -> Self {
        Self {
            inherent: Vec::new(),
        }
    }
}

impl Display for CallableCapabilities {
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
}

impl Default for PackageCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageCapabilities {
    pub fn new() -> Self {
        Self {
            callables: IndexMap::new(),
        }
    }
}

impl Display for PackageCapabilities {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Package:")?;
        indent = set_indentation(indent, 1);
        write!(indent, "\ncallables:")?;
        for (id, capabilities) in self.callables.iter() {
            indent = set_indentation(indent, 2);
            write!(indent, "\nid: {id}")?;
            indent = set_indentation(indent, 3);
            match capabilities {
                None => write!(indent, "\nNone")?,
                Some(c) => write!(indent, "{c}")?,
            }
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

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use fasteval::{Compiler, Evaler};
use std::collections::BTreeMap;

pub struct CompiledExpression {
    expression: String,
    name: String,
    instruction: fasteval::Instruction,
    slab: fasteval::Slab,
}

impl CompiledExpression {
    pub(crate) fn from_string(expression: &str, name: &str) -> super::Result<Self> {
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();

        let parsed = parser.parse(expression, &mut slab.ps)?;

        let instruction = parsed.from(&slab.ps).compile(&slab.ps, &mut slab.cs);

        Ok(Self {
            expression: expression.to_string(),
            name: name.to_string(),
            instruction,
            slab,
        })
    }

    pub fn evaluate(&self, context: &mut BTreeMap<String, f64>) -> Result<f64, fasteval::Error> {
        self.instruction.eval(&self.slab, context)
    }
}

impl std::fmt::Debug for CompiledExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CompiledExpression: {} = {}", self.name, self.expression)
    }
}

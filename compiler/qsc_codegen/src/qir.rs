// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod rir_utils;
#[cfg(test)]
mod tests;

use qsc_rir::rir;

trait ToQir<T> {
    fn to_qir(&self, program: &rir::Program) -> T;
}

impl ToQir<String> for rir::Literal {
    fn to_qir(&self, _program: &rir::Program) -> String {
        match self {
            rir::Literal::Bool(b) => format!("i1 {b}"),
            rir::Literal::Double(d) => {
                if (d.floor() - d.ceil()).abs() < f64::EPSILON {
                    // The value is a whole number, which requires at least one decimal point
                    // to differentiate it from an integer value.
                    format!("double {d:.1}")
                } else {
                    format!("double {d}")
                }
            }
            rir::Literal::Integer(i) => format!("i64 {i}"),
            rir::Literal::Pointer => "i8* null".to_string(),
            rir::Literal::Qubit(q) => format!("%Qubit* inttoptr (i64 {q} to %Qubit*)"),
            rir::Literal::Result(r) => format!("%Result* inttoptr (i64 {r} to %Result*)"),
        }
    }
}

impl ToQir<String> for rir::Ty {
    fn to_qir(&self, _program: &rir::Program) -> String {
        match self {
            rir::Ty::Boolean => "i1".to_string(),
            rir::Ty::Double => "double".to_string(),
            rir::Ty::Integer => "i64".to_string(),
            rir::Ty::Pointer => "i8*".to_string(),
            rir::Ty::Qubit => "%Qubit*".to_string(),
            rir::Ty::Result => "%Result*".to_string(),
        }
    }
}

impl ToQir<String> for Option<rir::Ty> {
    fn to_qir(&self, program: &rir::Program) -> String {
        match self {
            Some(ty) => ToQir::<String>::to_qir(ty, program),
            None => "void".to_string(),
        }
    }
}

impl ToQir<String> for rir::VariableId {
    fn to_qir(&self, _program: &rir::Program) -> String {
        format!("%var_{}", self.0)
    }
}

impl ToQir<String> for rir::Variable {
    fn to_qir(&self, program: &rir::Program) -> String {
        format!(
            "{} %var_{}",
            ToQir::<String>::to_qir(&self.ty, program),
            ToQir::<String>::to_qir(&self.variable_id, program)
        )
    }
}

impl ToQir<String> for rir::Value {
    fn to_qir(&self, program: &rir::Program) -> String {
        match self {
            rir::Value::Literal(lit) => ToQir::<String>::to_qir(lit, program),
            rir::Value::Variable(var) => ToQir::<String>::to_qir(var, program),
        }
    }
}

impl ToQir<String> for rir::Instruction {
    fn to_qir(&self, program: &rir::Program) -> String {
        match self {
            rir::Instruction::Store(_, _) => unimplemented!("store should be removed by pass"),
            rir::Instruction::Call(call_id, args) => {
                let args = args
                    .iter()
                    .map(|arg| ToQir::<String>::to_qir(arg, program))
                    .collect::<Vec<_>>()
                    .join(", ");
                let callable = program.get_callable(*call_id);
                format!(
                    "  call {} @{}({})",
                    ToQir::<String>::to_qir(&callable.output_type, program),
                    callable.name,
                    args
                )
            }
            rir::Instruction::Jump(_) => todo!(),
            rir::Instruction::Branch(_, _, _) => todo!(),
            rir::Instruction::Add(_, _, _) => todo!(),
            rir::Instruction::Sub(_, _, _) => todo!(),
            rir::Instruction::Mul(_, _, _) => todo!(),
            rir::Instruction::Div(_, _, _) => todo!(),
            rir::Instruction::LogicalNot(_, _) => todo!(),
            rir::Instruction::LogicalAnd(_, _, _) => todo!(),
            rir::Instruction::LogicalOr(_, _, _) => todo!(),
            rir::Instruction::BitwiseNot(_, _) => todo!(),
            rir::Instruction::BitwiseAnd(_, _, _) => todo!(),
            rir::Instruction::BitwiseOr(_, _, _) => todo!(),
            rir::Instruction::BitwiseXor(_, _, _) => todo!(),
            rir::Instruction::Return => "ret void".to_string(),
        }
    }
}

impl ToQir<String> for rir::Callable {
    fn to_qir(&self, program: &rir::Program) -> String {
        let input_type = self
            .input_type
            .iter()
            .map(|t| ToQir::<String>::to_qir(t, program))
            .collect::<Vec<_>>()
            .join(", ");
        let output_type = ToQir::<String>::to_qir(&self.output_type, program);
        let signature = format!("{} @{}({})", output_type, self.name, input_type);
        let Some(entry_id) = self.body else {
            return format!(
                "declare {signature} {}",
                if self.name == "__quantum__qis__mz__body" {
                    // The mz callable is a special case that needs the irreversable attribute.
                    "#1"
                } else {
                    ""
                }
            );
        };
        // For now, assume a single block.
        let block = program.get_block(entry_id);
        let body = block
            .0
            .iter()
            .map(|instr| ToQir::<String>::to_qir(instr, program))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "define {signature} #0 {{\nblock_{}:\n{body}\n}}",
            entry_id.0
        )
    }
}

impl ToQir<String> for rir::Program {
    fn to_qir(&self, _program: &rir::Program) -> String {
        let callables = self
            .callables
            .iter()
            .map(|(_, callable)| ToQir::<String>::to_qir(callable, self))
            .collect::<Vec<_>>()
            .join("\n\n");
        let profile = match self.profile {
            rir::Profile::Base => "base_profile",
            rir::Profile::Adaptive => "adaptive_profile",
        };
        format!(
            include_str!("./qir/template.ll"),
            callables, profile, self.num_qubits, self.num_results
        )
    }
}

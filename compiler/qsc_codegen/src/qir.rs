// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_rir::{rir, utils::get_all_block_successors};

/// A trait for converting a type into QIR of type `T`.
/// This can be used to generate QIR strings or other representations.
pub trait ToQir<T> {
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
            "{} {}",
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
            rir::Instruction::Call(call_id, args, output) => {
                let args = args
                    .iter()
                    .map(|arg| ToQir::<String>::to_qir(arg, program))
                    .collect::<Vec<_>>()
                    .join(", ");
                let callable = program.get_callable(*call_id);
                if let Some(output) = output {
                    format!(
                        "  {} = call {} @{}({})",
                        ToQir::<String>::to_qir(&output.variable_id, program),
                        ToQir::<String>::to_qir(&callable.output_type, program),
                        callable.name,
                        args
                    )
                } else {
                    format!(
                        "  call {} @{}({})",
                        ToQir::<String>::to_qir(&callable.output_type, program),
                        callable.name,
                        args
                    )
                }
            }
            rir::Instruction::Jump(block_id) => {
                format!("  br label %{}", ToQir::<String>::to_qir(block_id, program))
            }
            rir::Instruction::Branch(cond, true_id, false_id) => {
                format!(
                    "  br {}, label %{}, label %{}",
                    ToQir::<String>::to_qir(cond, program),
                    ToQir::<String>::to_qir(true_id, program),
                    ToQir::<String>::to_qir(false_id, program)
                )
            }
            rir::Instruction::Add(_, _, _) => todo!(),
            rir::Instruction::Sub(_, _, _) => todo!(),
            rir::Instruction::Mul(_, _, _) => todo!(),
            rir::Instruction::Sdiv(_, _, _) => todo!(),
            rir::Instruction::LogicalNot(_, _) => todo!(),
            rir::Instruction::LogicalAnd(_, _, _) => todo!(),
            rir::Instruction::LogicalOr(_, _, _) => todo!(),
            rir::Instruction::BitwiseNot(_, _) => todo!(),
            rir::Instruction::BitwiseAnd(_, _, _) => todo!(),
            rir::Instruction::BitwiseOr(_, _, _) => todo!(),
            rir::Instruction::BitwiseXor(_, _, _) => todo!(),
            rir::Instruction::Return => "  ret void".to_string(),
            rir::Instruction::Srem(_, _, _) => todo!(),
            rir::Instruction::Shl(_, _, _) => todo!(),
            rir::Instruction::Ashr(_, _, _) => todo!(),
            rir::Instruction::Icmp(_, _, _, _) => todo!(),
            rir::Instruction::Phi(_, _) => todo!(),
        }
    }
}

impl ToQir<String> for rir::BlockId {
    fn to_qir(&self, _program: &rir::Program) -> String {
        format!("block_{}", self.0)
    }
}

impl ToQir<String> for rir::Block {
    fn to_qir(&self, program: &rir::Program) -> String {
        self.0
            .iter()
            .map(|instr| ToQir::<String>::to_qir(instr, program))
            .collect::<Vec<_>>()
            .join("\n")
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
        let Some(entry_id) = self.body else {
            return format!(
                "declare {output_type} @{}({input_type}){}",
                self.name,
                if self.call_type == rir::CallableType::Measurement {
                    // Measurement callables are a special case that needs the irreversable attribute.
                    " #1"
                } else {
                    ""
                }
            );
        };
        let mut body = String::new();
        let all_blocks = get_all_block_successors(entry_id, program);
        for block_id in all_blocks {
            let block = program.get_block(block_id);
            body.push_str(&format!(
                "{}:\n{}\n",
                ToQir::<String>::to_qir(&block_id, program),
                ToQir::<String>::to_qir(block, program)
            ));
        }
        assert!(
            input_type.is_empty(),
            "entry point should not have an input"
        );
        format!("define {output_type} @ENTRYPOINT__main() #0 {{\n{body}}}",)
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
        let profile = if self.config.is_base() {
            "base_profile"
        } else {
            "adaptive_profile"
        };
        format!(
            include_str!("./qir/template.ll"),
            callables, profile, self.num_qubits, self.num_results
        )
    }
}

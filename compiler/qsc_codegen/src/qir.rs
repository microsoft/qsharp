// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod instruction_tests;

#[cfg(test)]
mod tests;

use qsc_rir::{
    rir::{self, IntPredicate},
    utils::get_all_block_successors,
};

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

impl ToQir<String> for rir::IntPredicate {
    fn to_qir(&self, _program: &rir::Program) -> String {
        match self {
            rir::IntPredicate::Eq => "eq".to_string(),
            rir::IntPredicate::Ne => "ne".to_string(),
            rir::IntPredicate::Sgt => "sgt".to_string(),
            rir::IntPredicate::Sge => "sge".to_string(),
            rir::IntPredicate::Slt => "slt".to_string(),
            rir::IntPredicate::Sle => "sle".to_string(),
        }
    }
}

impl ToQir<String> for rir::Instruction {
    fn to_qir(&self, program: &rir::Program) -> String {
        match self {
            rir::Instruction::Add(lhs, rhs, variable) => {
                binop_to_qir("add", lhs, rhs, *variable, program)
            }
            rir::Instruction::Ashr(lhs, rhs, variable) => {
                binop_to_qir("ashr", lhs, rhs, *variable, program)
            }
            rir::Instruction::BitwiseAnd(lhs, rhs, variable) => {
                simple_bitwise_to_qir("and", lhs, rhs, *variable, program)
            }
            rir::Instruction::BitwiseNot(value, variable) => {
                bitwise_not_to_qir(value, *variable, program)
            }
            rir::Instruction::BitwiseOr(lhs, rhs, variable) => {
                simple_bitwise_to_qir("or", lhs, rhs, *variable, program)
            }
            rir::Instruction::BitwiseXor(lhs, rhs, variable) => {
                simple_bitwise_to_qir("xor", lhs, rhs, *variable, program)
            }
            rir::Instruction::Branch(cond, true_id, false_id) => {
                format!(
                    "  br {}, label %{}, label %{}",
                    ToQir::<String>::to_qir(cond, program),
                    ToQir::<String>::to_qir(true_id, program),
                    ToQir::<String>::to_qir(false_id, program)
                )
            }
            rir::Instruction::Call(call_id, args, output) => {
                call_to_qir(args, *call_id, *output, program)
            }
            rir::Instruction::LogicalAnd(lhs, rhs, variable) => {
                logical_binop_to_qir("and", lhs, rhs, *variable, program)
            }
            rir::Instruction::LogicalNot(value, variable) => {
                logical_not_to_qir(value, *variable, program)
            }
            rir::Instruction::LogicalOr(lhs, rhs, variable) => {
                logical_binop_to_qir("or", lhs, rhs, *variable, program)
            }
            rir::Instruction::Mul(lhs, rhs, variable) => {
                binop_to_qir("mul", lhs, rhs, *variable, program)
            }
            rir::Instruction::Icmp(op, lhs, rhs, variable) => {
                icmp_to_qir(*op, lhs, rhs, *variable, program)
            }
            rir::Instruction::Jump(block_id) => {
                format!("  br label %{}", ToQir::<String>::to_qir(block_id, program))
            }
            rir::Instruction::Phi(args, variable) => phi_to_qir(args, *variable, program),
            rir::Instruction::Return => "  ret void".to_string(),
            rir::Instruction::Sdiv(lhs, rhs, variable) => {
                binop_to_qir("sdiv", lhs, rhs, *variable, program)
            }
            rir::Instruction::Shl(lhs, rhs, variable) => {
                binop_to_qir("shl", lhs, rhs, *variable, program)
            }
            rir::Instruction::Srem(lhs, rhs, variable) => {
                binop_to_qir("srem", lhs, rhs, *variable, program)
            }
            rir::Instruction::Store(_, _) => unimplemented!("store should be removed by pass"),
            rir::Instruction::Sub(lhs, rhs, variable) => {
                binop_to_qir("sub", lhs, rhs, *variable, program)
            }
        }
    }
}

fn logical_not_to_qir(
    value: &rir::Value,
    variable: rir::Variable,
    program: &rir::Program,
) -> String {
    let value_ty = get_value_ty(value);
    let var_ty = get_variable_ty(variable);
    assert_eq!(
        value_ty, var_ty,
        "mismatched input/output types ({value_ty}, {var_ty}) for not"
    );
    assert_eq!(var_ty, "i1", "unsupported type {var_ty} for not");

    format!(
        "  {} = xor i1 {}, true",
        ToQir::<String>::to_qir(&variable.variable_id, program),
        get_value_as_str(value, program)
    )
}

fn logical_binop_to_qir(
    op: &str,
    lhs: &rir::Value,
    rhs: &rir::Value,
    variable: rir::Variable,
    program: &rir::Program,
) -> String {
    let lhs_ty = get_value_ty(lhs);
    let rhs_ty = get_value_ty(rhs);
    let var_ty = get_variable_ty(variable);
    assert_eq!(
        lhs_ty, rhs_ty,
        "mismatched input types ({lhs_ty}, {rhs_ty}) for {op}"
    );
    assert_eq!(
        lhs_ty, var_ty,
        "mismatched input/output types ({lhs_ty}, {var_ty}) for {op}"
    );
    assert_eq!(var_ty, "i1", "unsupported type {var_ty} for {op}");

    format!(
        "  {} = {op} {var_ty} {}, {}",
        ToQir::<String>::to_qir(&variable.variable_id, program),
        get_value_as_str(lhs, program),
        get_value_as_str(rhs, program)
    )
}

fn bitwise_not_to_qir(
    value: &rir::Value,
    variable: rir::Variable,
    program: &rir::Program,
) -> String {
    let value_ty = get_value_ty(value);
    let var_ty = get_variable_ty(variable);
    assert_eq!(
        value_ty, var_ty,
        "mismatched input/output types ({value_ty}, {var_ty}) for not"
    );
    assert_eq!(var_ty, "i64", "unsupported type {var_ty} for not");

    format!(
        "  {} = xor {var_ty} {}, -1",
        ToQir::<String>::to_qir(&variable.variable_id, program),
        get_value_as_str(value, program)
    )
}

fn call_to_qir(
    args: &[rir::Value],
    call_id: rir::CallableId,
    output: Option<rir::Variable>,
    program: &rir::Program,
) -> String {
    let args = args
        .iter()
        .map(|arg| ToQir::<String>::to_qir(arg, program))
        .collect::<Vec<_>>()
        .join(", ");
    let callable = program.get_callable(call_id);
    if let Some(output) = output {
        format!(
            "  {} = call {} @{}({args})",
            ToQir::<String>::to_qir(&output.variable_id, program),
            ToQir::<String>::to_qir(&callable.output_type, program),
            callable.name
        )
    } else {
        format!(
            "  call {} @{}({args})",
            ToQir::<String>::to_qir(&callable.output_type, program),
            callable.name
        )
    }
}

fn icmp_to_qir(
    op: IntPredicate,
    lhs: &rir::Value,
    rhs: &rir::Value,
    variable: rir::Variable,
    program: &rir::Program,
) -> String {
    assert_eq!(get_value_ty(lhs), get_value_ty(rhs));
    let ty = get_value_ty(lhs);
    assert_eq!(ty, "i1");
    format!(
        "  {} = icmp {} {ty} {}, {}",
        ToQir::<String>::to_qir(&variable.variable_id, program),
        ToQir::<String>::to_qir(&op, program),
        get_value_as_str(lhs, program),
        get_value_as_str(rhs, program)
    )
}

fn binop_to_qir(
    op: &str,
    lhs: &rir::Value,
    rhs: &rir::Value,
    variable: rir::Variable,
    program: &rir::Program,
) -> String {
    let lhs_ty = get_value_ty(lhs);
    let rhs_ty = get_value_ty(rhs);
    let var_ty = get_variable_ty(variable);
    assert_eq!(
        lhs_ty, rhs_ty,
        "mismatched input types ({lhs_ty}, {rhs_ty}) for {op}"
    );
    assert_eq!(
        lhs_ty, var_ty,
        "mismatched input/output types ({lhs_ty}, {var_ty}) for {op}"
    );
    assert_eq!(var_ty, "i64", "unsupported type {var_ty} for {op}");

    format!(
        "  {} = {op} {var_ty} {}, {}",
        ToQir::<String>::to_qir(&variable.variable_id, program),
        get_value_as_str(lhs, program),
        get_value_as_str(rhs, program)
    )
}

fn simple_bitwise_to_qir(
    op: &str,
    lhs: &rir::Value,
    rhs: &rir::Value,
    variable: rir::Variable,
    program: &rir::Program,
) -> String {
    let lhs_ty = get_value_ty(lhs);
    let rhs_ty = get_value_ty(rhs);
    let var_ty = get_variable_ty(variable);
    assert_eq!(
        lhs_ty, rhs_ty,
        "mismatched input types ({lhs_ty}, {rhs_ty}) for {op}"
    );
    assert_eq!(
        lhs_ty, var_ty,
        "mismatched input/output types ({lhs_ty}, {var_ty}) for {op}"
    );
    assert_eq!(var_ty, "i64", "unsupported type {var_ty} for {op}");

    format!(
        "  {} = {op} {var_ty} {}, {}",
        ToQir::<String>::to_qir(&variable.variable_id, program),
        get_value_as_str(lhs, program),
        get_value_as_str(rhs, program)
    )
}

fn phi_to_qir(
    args: &[(rir::Value, rir::BlockId)],
    variable: rir::Variable,
    program: &rir::Program,
) -> String {
    assert!(
        !args.is_empty(),
        "phi instruction should have at least one argument"
    );
    let var_ty = get_variable_ty(variable);
    let args = args
        .iter()
        .map(|(arg, block_id)| {
            let arg_ty = get_value_ty(arg);
            assert_eq!(
                arg_ty, var_ty,
                "mismatched types ({var_ty} [... {arg_ty}]) for phi"
            );
            format!(
                "[{}, %{}]",
                get_value_as_str(arg, program),
                ToQir::<String>::to_qir(block_id, program)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "  {} = phi {var_ty} {args}",
        ToQir::<String>::to_qir(&variable.variable_id, program)
    )
}

fn get_value_as_str(value: &rir::Value, program: &rir::Program) -> String {
    match value {
        rir::Value::Literal(lit) => match lit {
            rir::Literal::Bool(b) => format!("{b}"),
            rir::Literal::Double(d) => {
                if (d.floor() - d.ceil()).abs() < f64::EPSILON {
                    // The value is a whole number, which requires at least one decimal point
                    // to differentiate it from an integer value.
                    format!("{d:.1}")
                } else {
                    format!("{d}")
                }
            }
            rir::Literal::Integer(i) => format!("{i}"),
            rir::Literal::Pointer => "null".to_string(),
            rir::Literal::Qubit(q) => format!("{q}"),
            rir::Literal::Result(r) => format!("{r}"),
        },
        rir::Value::Variable(var) => ToQir::<String>::to_qir(&var.variable_id, program),
    }
}

fn get_value_ty(lhs: &rir::Value) -> &str {
    match lhs {
        rir::Value::Literal(lit) => match lit {
            rir::Literal::Integer(_) => "i64",
            rir::Literal::Bool(_) => "i1",
            rir::Literal::Double(_) => "f64",
            rir::Literal::Qubit(_) => "%Qubit*",
            rir::Literal::Result(_) => "%Result*",
            rir::Literal::Pointer => "i8*",
        },
        rir::Value::Variable(var) => get_variable_ty(*var),
    }
}

fn get_variable_ty(variable: rir::Variable) -> &'static str {
    match variable.ty {
        rir::Ty::Integer => "i64",
        rir::Ty::Boolean => "i1",
        rir::Ty::Double => "f64",
        rir::Ty::Qubit => "%Qubit*",
        rir::Ty::Result => "%Result*",
        rir::Ty::Pointer => "i8*",
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

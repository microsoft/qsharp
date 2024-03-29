// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indenter::{indented, Indented};
use qsc_data_structures::index_map::IndexMap;
use std::fmt::{self, Debug, Display, Formatter, Write};

/// The root of the RIR.
#[derive(Default)]
pub struct Program {
    pub blocks: IndexMap<BlockId, Block>,
    pub callables: IndexMap<CallableId, Callable>,
    pub entry: CallableId,
    pub config: Config,
    pub num_qubits: u32,
    pub num_results: u32,
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Program:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nentry: {}", self.entry.0)?;
        write!(indent, "\ncallables:")?;
        indent = set_indentation(indent, 2);
        for (callable_id, callable) in self.callables.iter() {
            write!(indent, "\nCallable {}: {}", callable_id.0, callable)?;
        }
        indent = set_indentation(indent, 1);
        write!(indent, "\nblocks:")?;
        indent = set_indentation(indent, 2);
        for (block_id, block) in self.blocks.iter() {
            write!(indent, "\nBlock {}: {}", block_id.0, block)?;
        }
        indent = set_indentation(indent, 1);
        write!(indent, "\nconfig: {}", self.config)?;
        write!(indent, "\nnum_qubits: {}", self.num_qubits)?;
        write!(indent, "\nnum_results: {}", self.num_results)?;
        Ok(())
    }
}

impl Program {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get_callable(&self, id: CallableId) -> &Callable {
        self.callables.get(id).expect("callable should be present")
    }

    #[must_use]
    pub fn get_block(&self, id: BlockId) -> &Block {
        self.blocks.get(id).expect("block should be present")
    }
}

#[derive(Default)]
pub struct Config {
    pub remap_qubits_on_reuse: bool,
    pub defer_measurements: bool,
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Config:",)?;
        indent = set_indentation(indent, 1);
        write!(
            indent,
            "\nremap_qubits_on_reuse: {}",
            self.remap_qubits_on_reuse
        )?;
        write!(indent, "\ndefer_measurements: {}", self.defer_measurements)?;
        Ok(())
    }
}

impl Config {
    #[must_use]
    pub fn is_base(&self) -> bool {
        self.remap_qubits_on_reuse || self.defer_measurements
    }
}

/// A unique identifier for a block in a RIR program.
#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct BlockId(pub u32);

impl From<BlockId> for usize {
    fn from(id: BlockId) -> usize {
        id.0 as usize
    }
}

impl From<usize> for BlockId {
    fn from(id: usize) -> Self {
        Self(id.try_into().expect("block id should fit into u32"))
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Block:",)?;
        if self.0.is_empty() {
            write!(indent, " <EMPTY>")?;
        } else {
            indent = set_indentation(indent, 1);
            for instruction in &self.0 {
                write!(indent, "\n{instruction}")?;
            }
        }
        Ok(())
    }
}

/// A block is a collection of instructions.
#[derive(Default)]
pub struct Block(pub Vec<Instruction>);

/// A unique identifier for a callable in a RIR program.
#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct CallableId(pub u32);

impl From<CallableId> for usize {
    fn from(id: CallableId) -> Self {
        id.0 as Self
    }
}

impl From<usize> for CallableId {
    fn from(id: usize) -> Self {
        Self(id.try_into().expect("callable id should fit into u32"))
    }
}

/// A callable.
pub struct Callable {
    /// The name of the callable.
    pub name: String,
    /// The input type of the callable.
    pub input_type: Vec<Ty>,
    /// The output type of the callable.
    pub output_type: Option<Ty>,
    /// The callable body.
    /// N.B. `None` bodys represent an intrinsic.
    pub body: Option<BlockId>,
    /// What type of callable this is.
    pub call_type: CallableType,
}

impl Display for Callable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Callable:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nname: {}", self.name)?;
        write!(indent, "\ncall_type: {}", self.call_type)?;
        write!(indent, "\ninput_type: ")?;
        if self.input_type.is_empty() {
            write!(indent, " <VOID>")?;
        } else {
            indent = set_indentation(indent, 2);
            for (index, ty) in self.input_type.iter().enumerate() {
                write!(indent, "\n[{index}]: {ty}")?;
            }
            indent = set_indentation(indent, 1);
        }
        write!(indent, "\noutput_type: ")?;
        if let Some(output_type) = &self.output_type {
            write!(indent, " {output_type}")?;
        } else {
            write!(indent, " <VOID>")?;
        }
        write!(indent, "\nbody: ")?;
        if let Some(body_block_id) = self.body {
            write!(indent, " {}", body_block_id.0)?;
        } else {
            write!(indent, " <NONE>")?;
        }
        Ok(())
    }
}

/// The type of callable.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CallableType {
    Measurement,
    Readout,
    OutputRecording,
    Regular,
}
pub enum IntPredicate {
    Eq,
    Ne,
    Slt,
    Sle,
    Sgt,
    Sge,
}

impl Display for CallableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            Self::Measurement => write!(f, "Measurement")?,
            Self::Readout => write!(f, "Readout")?,
            Self::OutputRecording => write!(f, "OutputRecording")?,
            Self::Regular => write!(f, "Regular")?,
        };
        Ok(())
    }
}

pub enum Instruction {
    Store(Value, Variable),
    Call(CallableId, Vec<Value>, Option<Variable>),
    Jump(BlockId),
    Branch(Value, BlockId, BlockId),
    Add(Value, Value, Variable),
    Sub(Value, Value, Variable),
    Mul(Value, Value, Variable),
    Sdiv(Value, Value, Variable),
    Srem(Value, Value, Variable),
    Shl(Value, Value, Variable),
    Ashr(Value, Value, Variable),
    Icmp(IntPredicate, Value, Value, Variable),
    LogicalNot(Value, Variable),
    LogicalAnd(Value, Value, Variable),
    LogicalOr(Value, Value, Variable),
    BitwiseNot(Value, Variable),
    BitwiseAnd(Value, Value, Variable),
    BitwiseOr(Value, Value, Variable),
    BitwiseXor(Value, Value, Variable),
    Phi(Vec<(Value, BlockId)>, Variable),
    Return,
}

impl Display for Instruction {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fn write_binary_instruction(
            f: &mut Formatter,
            instruction: &str,
            lhs: &Value,
            rhs: &Value,
            variable: &Variable,
        ) -> fmt::Result {
            let mut indent = set_indentation(indented(f), 0);
            write!(indent, "{variable} = {instruction} {lhs}, {rhs}")?;
            Ok(())
        }

        fn write_branch(
            f: &mut Formatter,
            condition: &Value,
            if_true: BlockId,
            if_false: BlockId,
        ) -> fmt::Result {
            let mut indent = set_indentation(indented(f), 0);
            write!(indent, "Branch {condition}, {}, {}", if_true.0, if_false.0)?;
            Ok(())
        }

        fn write_call(
            f: &mut Formatter,
            callable_id: CallableId,
            args: &[Value],
            variable: &Option<Variable>,
        ) -> fmt::Result {
            let mut indent = set_indentation(indented(f), 0);
            if let Some(variable) = variable {
                write!(indent, "{variable} = ")?;
            }
            write!(indent, "Call id({}), args( ", callable_id.0)?;
            for arg in args {
                write!(indent, "{arg}, ")?;
            }
            write!(indent, ")")?;
            Ok(())
        }

        fn write_unary_instruction(
            f: &mut Formatter,
            instruction: &str,
            value: &Value,
            variable: &Variable,
        ) -> fmt::Result {
            let mut indent = set_indentation(indented(f), 0);
            write!(indent, "{variable} = {instruction} {value}")?;
            Ok(())
        }

        match &self {
            Self::Store(value, variable) => write_unary_instruction(f, "Store", value, variable)?,
            Self::Jump(block_id) => write!(f, "Jump({})", block_id.0)?,
            Self::Call(callable_id, args, variable) => write_call(f, *callable_id, args, variable)?,
            Self::Branch(condition, if_true, if_false) => {
                write_branch(f, condition, *if_true, *if_false)?;
            }
            Self::Add(lhs, rhs, variable) => {
                write_binary_instruction(f, "Add", lhs, rhs, variable)?;
            }
            Self::Sub(lhs, rhs, variable) => {
                write_binary_instruction(f, "Sub", lhs, rhs, variable)?;
            }
            Self::Mul(lhs, rhs, variable) => {
                write_binary_instruction(f, "Mul", lhs, rhs, variable)?;
            }
            Self::Div(lhs, rhs, variable) => {
                write_binary_instruction(f, "Div", lhs, rhs, variable)?;
            }
            Self::LogicalNot(value, variable) => {
                write_unary_instruction(f, "LogicalNot", value, variable)?;
            }
            Self::LogicalAnd(lhs, rhs, variable) => {
                write_binary_instruction(f, "LogicalAnd", lhs, rhs, variable)?;
            }
            Self::LogicalOr(lhs, rhs, variable) => {
                write_binary_instruction(f, "LogicalOr", lhs, rhs, variable)?;
            }
            Self::BitwiseNot(value, variable) => {
                write_unary_instruction(f, "BitwiseNot", value, variable)?;
            }
            Self::BitwiseAnd(lhs, rhs, variable) => {
                write_binary_instruction(f, "BitwiseAnd", lhs, rhs, variable)?;
            }
            Self::BitwiseOr(lhs, rhs, variable) => {
                write_binary_instruction(f, "BitwiseOr", lhs, rhs, variable)?;
            }
            Self::BitwiseXor(lhs, rhs, variable) => {
                write_binary_instruction(f, "BitwiseXor", lhs, rhs, variable)?;
            }
            Self::Return => write!(f, "Return")?,
        };
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
pub struct VariableId(pub u32);

pub struct Variable {
    pub variable_id: VariableId,
    pub ty: Ty,
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Variable({}, {})", self.variable_id.0, self.ty)?;
        Ok(())
    }
}

pub enum Ty {
    Qubit,
    Result,
    Boolean,
    Integer,
    Double,
    Pointer,
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            Self::Qubit => write!(f, "Qubit")?,
            Self::Result => write!(f, "Result")?,
            Self::Boolean => write!(f, "Boolean")?,
            Self::Integer => write!(f, "Integer")?,
            Self::Double => write!(f, "Double")?,
            Self::Pointer => write!(f, "Pointer")?,
        };
        Ok(())
    }
}

pub enum Value {
    Literal(Literal),
    Variable(Variable),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Variable(variable) => write!(f, "{variable}"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Literal {
    Qubit(u32),
    Result(u32),
    Bool(bool),
    Integer(i64),
    Double(f64),
    Pointer,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            Self::Qubit(id) => write!(f, "Qubit({id})")?,
            Self::Result(id) => write!(f, "Result({id})")?,
            Self::Bool(b) => write!(f, "Bool({b})")?,
            Self::Integer(i) => write!(f, "Integer({i})")?,
            Self::Double(d) => write!(f, "Double({d})")?,
            Self::Pointer => write!(f, "Pointer")?,
        };
        Ok(())
    }
}

fn set_indentation<'a, 'b>(
    indent: Indented<'a, Formatter<'b>>,
    level: usize,
) -> Indented<'a, Formatter<'b>> {
    match level {
        0 => indent.with_str(""),
        1 => indent.with_str("    "),
        2 => indent.with_str("        "),
        _ => unimplemented!("intentation level not supported"),
    }
}

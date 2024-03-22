// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;

/// The root of the RIR.
pub struct Program {
    pub blocks: IndexMap<BlockId, Block>,
    pub callables: IndexMap<CallableId, Callable>,
    pub entry: CallableId,
}

/// A unique identifier for a block in a RIR program.
#[derive(Clone, Copy, Default)]
pub struct BlockId(u32);

/// A block is a collection of instructions.
pub struct Block(pub Vec<Instruction>);

/// A unique identifier for a callable in a RIR program.
#[derive(Clone, Copy, Default)]
pub struct CallableId(u32);

/// A callable.
pub struct Callable {
    /// The ID of the callable.
    pub id: CallableId,
    /// The name of the callable.
    pub name: String,
    /// The input type of the callable.
    pub input_type: Vec<Ty>,
    /// The output type of the callable.
    pub output_type: Option<Ty>,
    /// The callable body.
    /// N.B. `None` bodys represent an intrinsic.
    pub body: Option<BlockId>,
}

pub enum Instruction {
    Store(Variable, Value),
    Call(CallableId, Vec<Value>),
    Jump(BlockId),
    Branch(Value, BlockId, BlockId),
    Add(Value, Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Div(Value, Value),
    LogicalNot(Value),
    LogicalAnd(Value, Value),
    LogicalOr(Value, Value),
    BitwiseNot(Value),
    BitwiseAnd(Value, Value),
    BitwiseOr(Value, Value),
    BitwiseXor(Value, Value),
}

#[derive(Clone, Copy, Default)]
pub struct VariableId(u32);

pub struct Variable {
    pub variable_id: VariableId,
    pub ty: Ty,
}

pub enum Ty {
    Qubit,
    Result,
    Boolean,
    Integer,
    Double,
}

pub enum Value {
    Literal(Literal),
    Variable(Variable),
}

#[derive(Clone, Copy)]
pub enum Literal {
    Qubit(u32),
    Result(u32),
    Bool(bool),
    Integer(i64),
    Double(f64),
}

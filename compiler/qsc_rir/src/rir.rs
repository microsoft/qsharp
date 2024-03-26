// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;

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

impl Config {
    #[must_use]
    pub fn is_base(&self) -> bool {
        self.remap_qubits_on_reuse || self.defer_measurements
    }
}

/// A unique identifier for a block in a RIR program.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct BlockId(pub u32);

impl From<BlockId> for usize {
    fn from(id: BlockId) -> usize {
        id.0 as usize
    }
}

/// A block is a collection of instructions.
#[derive(Default)]
pub struct Block(pub Vec<Instruction>);

/// A unique identifier for a callable in a RIR program.
#[derive(Clone, Copy, Default)]
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
}

pub enum Instruction {
    Store(Value, Variable),
    Call(CallableId, Vec<Value>, Option<Variable>),
    Jump(BlockId),
    Branch(Value, BlockId, BlockId),
    Add(Value, Value, Variable),
    Sub(Value, Value, Variable),
    Mul(Value, Value, Variable),
    Div(Value, Value, Variable),
    LogicalNot(Value, Variable),
    LogicalAnd(Value, Value, Variable),
    LogicalOr(Value, Value, Variable),
    BitwiseNot(Value, Variable),
    BitwiseAnd(Value, Value, Variable),
    BitwiseOr(Value, Value, Variable),
    BitwiseXor(Value, Value, Variable),
    Return,
}

#[derive(Clone, Copy, Default)]
pub struct VariableId(pub u32);

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
    Pointer,
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
    Pointer,
}

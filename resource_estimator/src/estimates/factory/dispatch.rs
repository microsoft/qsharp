use crate::estimates::{ErrorCorrection, Factory, FactoryBuilder};
use std::borrow::Cow;

/// Implements `Factory` to combine two factories for dispatching
///
/// It requires that all factories define the same code parameter type.
#[derive(Clone)]
pub enum FactoryDispatch2<F1, F2> {
    F1(F1),
    F2(F2),
}

impl<F1: Factory, F2: Factory<Parameter = F1::Parameter>> Factory for FactoryDispatch2<F1, F2> {
    type Parameter = F1::Parameter;

    fn physical_qubits(&self) -> u64 {
        match self {
            Self::F1(f) => f.physical_qubits(),
            Self::F2(f) => f.physical_qubits(),
        }
    }

    fn duration(&self) -> u64 {
        match self {
            Self::F1(f) => f.duration(),
            Self::F2(f) => f.duration(),
        }
    }

    fn num_output_states(&self) -> u64 {
        match self {
            Self::F1(f) => f.num_output_states(),
            Self::F2(f) => f.num_output_states(),
        }
    }

    fn max_code_parameter(&self) -> Option<Cow<Self::Parameter>> {
        match self {
            Self::F1(f) => f.max_code_parameter(),
            Self::F2(f) => f.max_code_parameter(),
        }
    }
}

/// Implements `FactoryBuilder` to combine two factory builders for dispatching
///
/// It will use `FactoryDispatch2` as factory type for the builder.
pub struct BuilderDispatch2<B1, B2> {
    b1: B1,
    b2: B2,
}

impl<B1: Default, B2: Default> Default for BuilderDispatch2<B1, B2> {
    fn default() -> Self {
        Self {
            b1: B1::default(),
            b2: B2::default(),
        }
    }
}

impl<E: ErrorCorrection, B1: FactoryBuilder<E>, B2: FactoryBuilder<E>> FactoryBuilder<E>
    for BuilderDispatch2<B1, B2>
{
    type Factory = FactoryDispatch2<B1::Factory, B2::Factory>;

    fn find_factories(
        &self,
        ftp: &E,
        qubit: &std::rc::Rc<E::Qubit>,
        magic_state_type: usize,
        output_error_rate: f64,
        max_code_parameter: &E::Parameter,
    ) -> Vec<Cow<Self::Factory>> {
        match magic_state_type {
            0 => self
                .b1
                .find_factories(
                    ftp,
                    qubit,
                    magic_state_type,
                    output_error_rate,
                    max_code_parameter,
                )
                .into_iter()
                .map(|f| Cow::Owned(FactoryDispatch2::F1(f.into_owned())))
                .collect(),
            1 => self
                .b2
                .find_factories(
                    ftp,
                    qubit,
                    magic_state_type,
                    output_error_rate,
                    max_code_parameter,
                )
                .into_iter()
                .map(|f| Cow::Owned(FactoryDispatch2::F2(f.into_owned())))
                .collect(),
            _ => unreachable!("factory builder only has two magic state types"),
        }
    }

    /// Since this builder dispatch is for two factory builders, it can support
    /// two different magic state types.
    fn num_magic_state_types(&self) -> usize {
        2
    }
}

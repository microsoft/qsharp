// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export
    GeneratorIndex,
    GeneratorSystem,
    EvolutionGenerator,
    HTermToGenIdx,
    HTermsToGenSys,
    HTermsToGenIdx;

import Std.Math.AbsD;

/// # Summary
/// Represents a single primitive term in the set of all dynamical generators, e.g.
/// Hermitian operators, for which there exists a map from that generator
/// to time-evolution by that generator, through an evolution set function.
///
/// The first element
/// (Int[], Double[]) is indexes that single term -- For instance, the Pauli string
/// XXY with coefficient 0.5 would be indexed by ([1,1,2], [0.5]). Alternatively,
/// Hamiltonians parameterized by a continuous variable, such as X cos φ + Y sin φ,
/// might for instance be represented by ([], [φ]). The second
/// element indexes the subsystem on which the generator acts on.
///
/// # Remarks
/// > [!WARNING]
/// > The interpretation of an `GeneratorIndex` is not defined except
/// > with reference to a particular set of generators.
///
/// # Example
/// Using Pauli evolution set, the operator
/// $\pi X_2 X_5 Y_9$ is represented as:
/// ```qsharp
/// let index = new GeneratorIndex {
///     Term = ([1, 1, 2], [PI()]),
///     Subsystem = [2, 5, 9]
/// };
/// ```
struct GeneratorIndex {
    Term : (Int[], Double[]),
    Subsystem : Int[],
}

/// # Summary
/// Represents a collection of `GeneratorIndex`es.
///
/// We iterate over this
/// collection using a single-index integer, and the size of the
/// collection is assumed to be known.
struct GeneratorSystem {
    NumEntries : Int,
    EntryAt : (Int -> GeneratorIndex)
}

/// # Summary
/// Represents a dynamical generator as a set of simulatable gates and
/// an expansion in terms of that basis.
///
/// Last parameter for number of terms.
struct EvolutionGenerator {
    EvolutionSet : GeneratorIndex -> (Double, Qubit[]) => Unit is Adj + Ctl,
    System : GeneratorSystem
}

/// # Summary
/// Converts a Hamiltonian term to a GeneratorIndex.
///
/// # Input
/// ## term
/// Input data in `(Int[], Double[])` format.
/// ## termType
/// Additional information added to GeneratorIndex.
///
/// # Output
/// A GeneratorIndex representing a Hamiltonian term represented by `term`,
/// together with additional information added by `termType`.
function HTermToGenIdx(term : (Int[], Double[]), termType : Int[]) : GeneratorIndex {
    let (idxFermions, coeff) = term;
    new GeneratorIndex { Term = (termType, coeff), Subsystem = idxFermions }
}


/// # Summary
/// Converts an index to a Hamiltonian term in `(Int[], Double[])[]` data format to a GeneratorIndex.
///
/// # Input
/// ## data
/// Input data in `(Int[], Double[])[]` format.
/// ## termType
/// Additional information added to GeneratorIndex.
/// ## idx
/// Index to a term of the Hamiltonian
///
/// # Output
/// A GeneratorIndex representing a Hamiltonian term represented by `data[idx]`,
/// together with additional information added by `termType`.
function HTermsToGenIdx(data : (Int[], Double[])[], termType : Int[], idx : Int) : GeneratorIndex {
    return HTermToGenIdx(data[idx], termType);
}


/// # Summary
/// Converts a Hamiltonian in `(Int[], Double[])[]` data format to a GeneratorSystem.
///
/// # Input
/// ## data
/// Input data in `(Int[], Double[])[]` format.
/// ## termType
/// Additional information added to GeneratorIndex.
///
/// # Output
/// A GeneratorSystem representing a Hamiltonian represented by the input `data`.
function HTermsToGenSys(data : (Int[], Double[])[], termType : Int[]) : GeneratorSystem {
    new GeneratorSystem { NumEntries = Length(data), EntryAt = HTermsToGenIdx(data, termType, _) }
}


/// # Summary
/// Checks whether a `Double` number is not approximately zero.
///
/// # Input
/// ## number
/// Number to be checked
///
/// # Output
/// Returns true if `number` has an absolute value greater than `1e-15`.
function IsNotZero(number : Double) : Bool {
    AbsD(number) > 1e-15
}

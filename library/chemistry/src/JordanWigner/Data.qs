// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export JWOptimizedHTerms;
export JordanWignerInputState;
export JordanWignerEncodingData;

import Std.Math.*;

import Generators.EvolutionGenerator;

/// # Summary
/// Format for a four term Jordan-Wigner optimized Hamiltonian.
/// The meaning of the data represented is determined by the algorithm that receives it.
struct JWOptimizedHTerms {
    HTerm0 : (Int[], Double[])[],
    HTerm1 : (Int[], Double[])[],
    HTerm2 : (Int[], Double[])[],
    HTerm3 : (Int[], Double[])[],
}

/// # Summary
/// Represents preparation of the initial state
/// The meaning of the data represented is determined by the algorithm that receives it.
struct JordanWignerInputState {
    Amplitude : Complex,
    FermionIndices : Int[],
}

/// # Summary
/// Format of data to represent all information for Hamiltonian simulation.
/// The meaning of the data represented is determined by the algorithm that receives it.
struct JordanWignerEncodingData {
    NumQubits : Int,
    Terms : JWOptimizedHTerms,
    InputState : (Int, JordanWignerInputState[]),
    EnergyOffset : Double,
}

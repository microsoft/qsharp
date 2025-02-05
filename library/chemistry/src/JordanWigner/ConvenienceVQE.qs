// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export
    EstimateEnergyWrapper,
    EstimateEnergy;

import JordanWigner.JordanWignerEvolutionSet.JordanWignerGeneratorSystem;
import JordanWigner.JordanWignerVQE.EstimateTermExpectation;
import JordanWigner.JordanWignerVQE.ExpandedCoefficients;
import JordanWigner.JordanWignerVQE.MeasurementOperators;
import JordanWigner.StatePreparation.PrepareTrialState;
import JordanWigner.Utils.JordanWignerEncodingData;
import JordanWigner.Utils.JordanWignerInputState;
import JordanWigner.Utils.JWOptimizedHTerms;

/// # Summary
/// Estimates the energy of the molecule by summing the energy contributed by the individual Jordan-Wigner terms.
/// This convenience wrapper takes input in raw data types and converts them to the Jordan-Wigner encoding data type.
///
/// # Description
/// This operation implicitly relies on the spin up-down indexing convention.
///
/// # Input
/// ## jwHamiltonian
/// The Jordan-Wigner Hamiltonian, represented in simple data types rather than a struct.
/// ## nSamples
/// The number of samples to use for the estimation of the term expectations.
///
/// # Output
/// The estimated energy of the molecule
operation EstimateEnergyWrapper(jwHamiltonian : (Int, ((Int[], Double[])[], (Int[], Double[])[], (Int[], Double[])[], (Int[], Double[])[]), (Int, ((Double, Double), Int[])[]), Double), nSamples : Int) : Double {
    let (nQubits, jwTerms, inputState, energyOffset) = jwHamiltonian;
    let (hterm0, hterm1, hterm2, hterm3) = jwTerms;
    let jwTerms = new JWOptimizedHTerms { HTerm0 = hterm0, HTerm1 = hterm1, HTerm2 = hterm2, HTerm3 = hterm3 };
    let (inputState1, inputState2) = inputState;
    mutable jwInputState = [];
    for entry in inputState2 {
        let (amp, idicies) = entry;
        jwInputState += [new JordanWignerInputState { Amplitude = amp, FermionIndices = idicies }];
    }
    let inputState = (inputState1, jwInputState);
    let jwHamiltonian = new JordanWignerEncodingData {
        NumQubits = nQubits,
        Terms = jwTerms,
        InputState = inputState,
        EnergyOffset = energyOffset
    };
    return EstimateEnergy(jwHamiltonian, nSamples);
}

/// # Summary
/// Estimates the energy of the molecule by summing the energy contributed by the individual Jordan-Wigner terms.
///
/// # Description
/// This operation implicitly relies on the spin up-down indexing convention.
///
/// # Input
/// ## jwHamiltonian
/// The Jordan-Wigner Hamiltonian.
/// ## nSamples
/// The number of samples to use for the estimation of the term expectations.
///
/// # Output
/// The estimated energy of the molecule
operation EstimateEnergy(jwHamiltonian : JordanWignerEncodingData, nSamples : Int) : Double {
    // Initialize return value
    mutable energy = 0.;

    // Unpack information and qubit Hamiltonian terms
    let (nQubits, jwTerms, inputState, energyOffset) = jwHamiltonian!;

    // Loop over all qubit Hamiltonian terms
    let (nTerms, indexFunction) = (JordanWignerGeneratorSystem(jwTerms))!;

    for idxTerm in 0..nTerms - 1 {
        let term = indexFunction(idxTerm);
        let ((idxTermType, coeff), idxFermions) = term!;
        let termType = idxTermType[0];

        let ops = MeasurementOperators(nQubits, idxFermions, termType);
        let coeffs = ExpandedCoefficients(coeff, termType);

        // The private wrapper enables fast emulation during expectation estimation
        let inputStateUnitary = PrepareTrialState(inputState, _);

        let jwTermEnergy = EstimateTermExpectation(inputStateUnitary, ops, coeffs, nQubits, nSamples);
        energy += jwTermEnergy;
    }

    return energy + energyOffset;
}

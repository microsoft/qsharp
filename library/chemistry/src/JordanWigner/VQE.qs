// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export EstimateEnergyWrapper;
export EstimateEnergy;
export EstimateTermExpectation;
export EstimateFrequency;
export MeasurementOperators;
export ExpandedCoefficients;

import Std.Arrays.IndexRange;
import Std.Convert.IntAsDouble;
import Std.Math.AbsD;
import Std.Math.Complex;

import JordanWigner.EvolutionSet.JWGeneratorSystem;
import JordanWigner.StatePreparation.PrepareTrialState;
import JordanWigner.Data.JWEncodingData;
import JordanWigner.Data.JWInputState;
import JordanWigner.Data.JWOptimizedHTerms;

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
operation EstimateEnergyWrapper(
    jwHamiltonian : (
        Int,
        ((Int[], Double[])[], (Int[], Double[])[], (Int[], Double[])[], (Int[], Double[])[]),
        (Int, ((Double, Double), Int[])[]),
        Double
    ),
    nSamples : Int
) : Double {

    let (nQubits, jwTerms, inputState, energyOffset) = jwHamiltonian;
    let (hterm0, hterm1, hterm2, hterm3) = jwTerms;
    let jwTerms = new JWOptimizedHTerms { HTerm0 = hterm0, HTerm1 = hterm1, HTerm2 = hterm2, HTerm3 = hterm3 };
    let (inputState1, inputState2) = inputState;
    mutable jwInputState = [];
    for entry in inputState2 {
        let ((r, i), idicies) = entry;
        jwInputState += [new JWInputState { Amplitude = new Complex { Real = r, Imag = i }, FermionIndices = idicies }];
    }
    let inputState = (inputState1, jwInputState);
    let jwHamiltonian = new JWEncodingData {
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
operation EstimateEnergy(
    jwHamiltonian : JWEncodingData,
    nSamples : Int
) : Double {

    // Initialize return value
    mutable energy = 0.;

    let nQubits = jwHamiltonian.NumQubits;

    // Loop over all qubit Hamiltonian terms
    let generatorSystem = JWGeneratorSystem(jwHamiltonian.Terms);

    for idxTerm in 0..generatorSystem.NumEntries - 1 {
        let term = generatorSystem.EntryAt(idxTerm);
        let (idxTermType, coeff) = term.Term;
        let idxFermions = term.Subsystem;
        let termType = idxTermType[0];

        let ops = MeasurementOperators(nQubits, idxFermions, termType);
        let coeffs = ExpandedCoefficients(coeff, termType);

        // The private wrapper enables fast emulation during expectation estimation
        let inputStateUnitary = PrepareTrialState(jwHamiltonian.InputState, _);

        let jwTermEnergy = EstimateTermExpectation(inputStateUnitary, ops, coeffs, nQubits, nSamples);
        energy += jwTermEnergy;
    }

    return energy + jwHamiltonian.EnergyOffset;
}

/// # Summary
/// Computes the energy associated to a given Jordan-Wigner Hamiltonian term
///
/// # Description
/// This operation estimates the expectation value associated to each measurement operator and
/// multiplies it by the corresponding coefficient, using sampling.
/// The results are aggregated into a variable containing the energy of the Jordan-Wigner term.
///
/// # Input
/// ## inputStateUnitary
/// The unitary used for state preparation.
/// ## ops
/// The measurement operators of the Jordan-Wigner term.
/// ## coeffs
/// The coefficients of the Jordan-Wigner term.
/// ## nQubits
/// The number of qubits required to simulate the molecular system.
/// ## nSamples
/// The number of samples to use for the estimation of the term expectation.
///
/// # Output
/// The energy associated to the Jordan-Wigner term.
operation EstimateTermExpectation(
    inputStateUnitary : (Qubit[] => Unit),
    ops : Pauli[][],
    coeffs : Double[],
    nQubits : Int,
    nSamples : Int
) : Double {

    mutable jwTermEnergy = 0.;

    for i in IndexRange(coeffs) {
        // Only perform computation if the coefficient is significant enough
        if (AbsD(coeffs[i]) >= 1e-10) {
            // Compute expectation value using the fast frequency estimator, add contribution to Jordan-Wigner term energy
            let termExpectation = EstimateFrequency(inputStateUnitary, Measure(ops[i], _), nQubits, nSamples);
            jwTermEnergy += (2. * termExpectation - 1.) * coeffs[i];
        }
    }

    return jwTermEnergy;
}

/// # Summary
/// Given a preparation and measurement, estimates the frequency
/// with which that measurement succeeds (returns `Zero`) by
/// performing a given number of trials.
///
/// # Input
/// ## preparation
/// An operation $P$ that prepares a given state $\rho$ on
/// its input register.
/// ## measurement
/// An operation $M$ representing the measurement of interest.
/// ## nQubits
/// The number of qubits on which the preparation and measurement
/// each act.
/// ## nMeasurements
/// The number of times that the measurement should be performed
/// in order to estimate the frequency of interest.
///
/// # Output
/// An estimate $\hat{p}$ of the frequency with which
/// $M(P(\ket{00 \cdots 0}\bra{00 \cdots 0}))$ returns `Zero`,
/// obtained using the unbiased binomial estimator $\hat{p} =
/// n\_{\uparrow} / n\_{\text{measurements}}$, where $n\_{\uparrow}$ is
/// the number of `Zero` results observed.
///
/// This is particularly important on target machines which respect
/// physical limitations, such that probabilities cannot be measured.
operation EstimateFrequency(
    preparation : (Qubit[] => Unit),
    measurement : (Qubit[] => Result),
    nQubits : Int,
    nMeasurements : Int
) : Double {

    mutable nUp = 0;

    for idxMeasurement in 0..nMeasurements - 1 {
        use register = Qubit[nQubits];
        preparation(register);
        let result = measurement(register);

        if (result == Zero) {
            // NB!!!!! This reverses Zero and One to use conventions
            //         common in the QCVV community. That is confusing
            //         but is confusing with an actual purpose.
            nUp = nUp + 1;
        }

        // NB: We absolutely must reset here, since preparation()
        //     and measurement() can each use randomness internally.
        ResetAll(register);
    }

    return IntAsDouble(nUp) / IntAsDouble(nMeasurements);
}

/// # Summary
/// Computes all the measurement operators required to compute the expectation of a Jordan-Wigner term.
///
/// # Input
/// ## nQubits
/// The number of qubits required to simulate the molecular system.
/// ## indices
/// An array containing the indices of the qubit each Pauli operator is applied to.
/// ## termType
/// The type of the Jordan-Wigner term.
///
/// # Output
/// An array of measurement operators (each being an array of Pauli).
function MeasurementOperators(
    nQubits : Int,
    indices : Int[],
    termType : Int
) : Pauli[][] {

    // Compute the size and initialize the array of operators to be returned
    mutable nOps = 0;
    if (termType == 2) {
        nOps = 2;
    } elif (termType == 3) {
        nOps = 8;
    } else {
        nOps = 1;
    }

    mutable ops = Repeated([], nOps);

    // Z and ZZ terms
    if termType == 0 or termType == 1 {
        mutable op = Repeated(PauliI, nQubits);
        for idx in indices {
            op[idx] = PauliZ;
        }
        ops[0] = op;
    }

    // PQRS terms set operators between indices P and Q (resp R and S) to PauliZ
    elif termType == 3 {
        let compactOps = [
            [PauliX, PauliX, PauliX, PauliX],
            [PauliY, PauliY, PauliY, PauliY],
            [PauliX, PauliX, PauliY, PauliY],
            [PauliY, PauliY, PauliX, PauliX],
            [PauliX, PauliY, PauliX, PauliY],
            [PauliY, PauliX, PauliY, PauliX],
            [PauliY, PauliX, PauliX, PauliY],
            [PauliX, PauliY, PauliY, PauliX]
        ];

        for iOp in 0..7 {
            mutable compactOp = compactOps[iOp];

            mutable op = Repeated(PauliI, nQubits);
            for idx in IndexRange(indices) {
                op[indices[idx]] = compactOp[idx];
            }
            for i in indices[0] + 1..indices[1] - 1 {
                op[i] = PauliZ;
            }
            for i in indices[2] + 1..indices[3] - 1 {
                op[i] = PauliZ;
            }
            ops[iOp] = op;
        }
    }

    // Case of PQ and PQQR terms
    elif termType == 2 {
        let compactOps = [[PauliX, PauliX], [PauliY, PauliY]];

        for iOp in 0..1 {
            mutable compactOp = compactOps[iOp];

            mutable op = Repeated(PauliI, nQubits);

            let nIndices = Length(indices);
            op[indices[0]] = compactOp[0];
            op[indices[nIndices-1]] = compactOp[1];
            for i in indices[0] + 1..indices[nIndices - 1] - 1 {
                op[i] = PauliZ;
            }

            // Case of PQQR term
            if nIndices == 4 {
                op[indices[1]] = (indices[0] < indices[1] and indices[1] < indices[3]) ? PauliI | PauliZ;
            }
            ops[iOp] = op;
        }
    }

    return ops;
}

/// # Summary
/// Expands the compact representation of the Jordan-Wigner coefficients in order
/// to obtain a one-to-one mapping between these and Pauli terms.
///
/// # Input
/// ## coeff
/// An array of coefficients, as read from the Jordan-Wigner Hamiltonian data structure.
/// ## termType
/// The type of the Jordan-Wigner term.
///
/// # Output
/// Expanded arrays of coefficients, one per Pauli term.
function ExpandedCoefficients(coeff : Double[], termType : Int) : Double[] {
    // Compute the numbers of coefficients to return
    mutable nCoeffs = 0;
    if (termType == 2) {
        nCoeffs = 2;
    } elif (termType == 3) {
        nCoeffs = 8;
    } else {
        nCoeffs = 1;
    }

    mutable coeffs = Repeated(0.0, nCoeffs);

    // Return the expanded array of coefficients
    if termType == 0 or termType == 1 {
        coeffs[0] = coeff[0];
    } elif termType == 2 or termType == 3 {
        for i in 0..nCoeffs - 1 {
            coeffs[i] = coeff[i / 2];
        }
    }

    return coeffs;
}

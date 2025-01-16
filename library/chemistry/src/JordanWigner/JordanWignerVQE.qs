// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export
    EstimateTermExpectation,
    EstimateFrequency,
    MeasurementOperators,
    ExpandedCoefficients;

import Std.Arrays.IndexRange;
import Std.Convert.IntAsDouble;
import Std.Math.AbsD;

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
operation EstimateTermExpectation(inputStateUnitary : (Qubit[] => Unit), ops : Pauli[][], coeffs : Double[], nQubits : Int, nSamples : Int) : Double {
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
operation EstimateFrequency(preparation : (Qubit[] => Unit), measurement : (Qubit[] => Result), nQubits : Int, nMeasurements : Int) : Double {
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
        ApplyToEach(Reset, register);
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
function MeasurementOperators(nQubits : Int, indices : Int[], termType : Int) : Pauli[][] {
    // Compute the size and initialize the array of operators to be returned
    mutable nOps = 0;
    if (termType == 2) {
        nOps = 2;
    } elif (termType == 3) {
        nOps = 8;
    } else {
        nOps = 1;
    }

    mutable ops = [[], size = nOps];

    // Z and ZZ terms
    if termType == 0 or termType == 1 {
        mutable op = Repeated(PauliI, nQubits);
        for idx in indices {
            op w/= idx <- PauliZ;
        }
        ops w/= 0 <- op;
    }

    // PQRS terms set operators between indices P and Q (resp R and S) to PauliZ
    elif termType == 3 {
        let compactOps = [[PauliX, PauliX, PauliX, PauliX], [PauliY, PauliY, PauliY, PauliY], [PauliX, PauliX, PauliY, PauliY], [PauliY, PauliY, PauliX, PauliX], [PauliX, PauliY, PauliX, PauliY], [PauliY, PauliX, PauliY, PauliX], [PauliY, PauliX, PauliX, PauliY], [PauliX, PauliY, PauliY, PauliX]];

        for iOp in 0..7 {
            mutable compactOp = compactOps[iOp];

            mutable op = Repeated(PauliI, nQubits);
            for idx in IndexRange(indices) {
                op w/= indices[idx] <- compactOp[idx];
            }
            for i in indices[0] + 1..indices[1] - 1 {
                op w/= i <- PauliZ;
            }
            for i in indices[2] + 1..indices[3] - 1 {
                op w/= i <- PauliZ;
            }
            ops w/= iOp <- op;
        }
    }

    // Case of PQ and PQQR terms
    elif termType == 2 {
        let compactOps = [[PauliX, PauliX], [PauliY, PauliY]];

        for iOp in 0..1 {
            mutable compactOp = compactOps[iOp];

            mutable op = Repeated(PauliI, nQubits);

            let nIndices = Length(indices);
            op w/= indices[0] <- compactOp[0];
            op w/= indices[nIndices-1] <- compactOp[1];
            for i in indices[0] + 1..indices[nIndices - 1] - 1 {
                op w/= i <- PauliZ;
            }

            // Case of PQQR term
            if nIndices == 4 {
                op w/= indices[1] <- (indices[0] < indices[1] and indices[1] < indices[3]) ? PauliI | PauliZ;
            }
            ops w/= iOp <- op;
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

    mutable coeffs = [0.0, size = nCoeffs];

    // Return the expanded array of coefficients
    if termType == 0 or termType == 1 {
        coeffs w/= 0 <- coeff[0];
    } elif termType == 2 or termType == 3 {
        for i in 0..nCoeffs - 1 {
            coeffs w/= i <- coeff[i / 2];
        }
    }

    return coeffs;
}

// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export TrotterStepOracle;
export QubitizationOracle;
export OptimizedQubitizationOracle;

import Std.Convert.IntAsDouble;
import Std.Math.Ceiling;
import Std.Math.Lg;

import JordanWigner.BlockEncoding.JWBlockEncodingGeneratorSystem;
import JordanWigner.EvolutionSet.JWFermionEvolutionSet;
import JordanWigner.EvolutionSet.JWGeneratorSystem;
import JordanWigner.OptimizedBlockEncoding.JWOptimizedBlockEncoding;
import JordanWigner.OptimizedBlockEncoding.PauliBlockEncoding;
import JordanWigner.OptimizedBlockEncoding.QuantumWalkByQubitization;
import JordanWigner.Data.JordanWignerEncodingData;
import Trotterization.TrotterSimulationAlgorithm;
import Generators.EvolutionGenerator;

// Convenience functions for performing simulation.

/// # Summary
/// Returns Trotter step operation and the parameters necessary to run it.
///
/// # Input
/// ## jwHamiltonian
/// Hamiltonian described by `JordanWignerEncodingData` format.
/// ## trotterStepSize
/// Step size of Trotter integrator.
/// ## trotterOrder
/// Order of Trotter integrator.
///
/// # Output
/// A tuple where: `Int` is the number of qubits allocated,
/// `Double` is `1.0/trotterStepSize`, and the operation
/// is the Trotter step.
function TrotterStepOracle(
    jwHamiltonian : JordanWignerEncodingData,
    trotterStepSize : Double,
    trotterOrder : Int
) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let generatorSystem = JWGeneratorSystem(jwHamiltonian.Terms);
    let evolutionGenerator = new EvolutionGenerator { EvolutionSet = JWFermionEvolutionSet(), System = generatorSystem };
    let simulationAlgorithm = TrotterSimulationAlgorithm(trotterStepSize, trotterOrder);
    let oracle = simulationAlgorithm(trotterStepSize, evolutionGenerator, _);
    let nTargetRegisterQubits = jwHamiltonian.NumQubits;
    let rescaleFactor = 1.0 / trotterStepSize;
    return (nTargetRegisterQubits, (rescaleFactor, oracle));
}


function QubitizationOracleSeperatedRegisters(
    jwHamiltonian : JordanWignerEncodingData
) : ((Int, Int), (Double, ((Qubit[], Qubit[]) => Unit is Adj + Ctl))) {
    let generatorSystem = JWBlockEncodingGeneratorSystem(jwHamiltonian.Terms);
    let (oneNorm, blockEncodingReflection) = PauliBlockEncoding(generatorSystem);
    let nTargetRegisterQubits = jwHamiltonian.NumQubits;
    let nCtrlRegisterQubits = Ceiling(Lg(IntAsDouble(generatorSystem.NumEntries)));
    return ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, QuantumWalkByQubitization(blockEncodingReflection)));
}

/// # Summary
/// Returns Qubitization operation and the parameters necessary to run it.
///
/// # Input
/// ## jwHamiltonian
/// Hamiltonian described by `JordanWignerEncodingData` format.
///
/// # Output
/// A tuple where: `Int` is the number of qubits allocated,
/// `Double` is the one-norm of Hamiltonian coefficients, and the operation
/// is the Quantum walk created by Qubitization.
function QubitizationOracle(
    jwHamiltonian : JordanWignerEncodingData
) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, oracle)) = QubitizationOracleSeperatedRegisters(jwHamiltonian);
    let nQubits = nCtrlRegisterQubits + nTargetRegisterQubits;
    return (nQubits, (oneNorm, ApplyOracleOnRegisterParts(oracle, nTargetRegisterQubits, _)));
}


function OptimizedQubitizationOracleSeperatedRegisters(
    jwHamiltonian : JordanWignerEncodingData,
    targetError : Double
) : ((Int, Int), (Double, ((Qubit[], Qubit[]) => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, blockEncodingReflection)) = JWOptimizedBlockEncoding(targetError, jwHamiltonian.Terms, jwHamiltonian.NumQubits);
    return ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, QuantumWalkByQubitization(blockEncodingReflection)));
}


/// # Summary
/// Returns T-count optimized Qubitization operation
/// and the parameters necessary to run it.
///
/// # Input
/// ## jwHamiltonian
/// Hamiltonian described by `JordanWignerEncodingData` format.
/// ## targetError
/// Error of auxillary state preparation step.
///
/// # Output
/// A tuple where: `Int` is the number of qubits allocated,
/// `Double` is the one-norm of Hamiltonian coefficients, and the operation
/// is the Quantum walk created by Qubitization.
function OptimizedQubitizationOracle(
    jwHamiltonian : JordanWignerEncodingData,
    targetError : Double
) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, oracle)) = OptimizedQubitizationOracleSeperatedRegisters(jwHamiltonian, targetError);
    let nQubits = nCtrlRegisterQubits + nTargetRegisterQubits;
    return (nQubits, (oneNorm, ApplyOracleOnRegisterParts(oracle, nTargetRegisterQubits, _)));
}


operation ApplyOracleOnRegisterParts(
    oracle : ((Qubit[], Qubit[]) => Unit is Adj + Ctl),
    nSystemQubits : Int,
    allQubits : Qubit[]
) : Unit is Adj + Ctl {
    oracle(allQubits[nSystemQubits..Length(allQubits) - 1], allQubits[0..nSystemQubits - 1]);
}

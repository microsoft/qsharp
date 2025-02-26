// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export
    TrotterStepOracle,
    QubitizationOracle,
    OptimizedQubitizationOracle;

import Std.Convert.IntAsDouble;
import Std.Math.Ceiling;
import Std.Math.Lg;

import JordanWigner.JordanWignerBlockEncoding.JordanWignerBlockEncodingGeneratorSystem;
import JordanWigner.JordanWignerEvolutionSet.JordanWignerFermionEvolutionSet;
import JordanWigner.JordanWignerEvolutionSet.JordanWignerGeneratorSystem;
import JordanWigner.JordanWignerOptimizedBlockEncoding.JordanWignerOptimizedBlockEncoding;
import JordanWigner.JordanWignerOptimizedBlockEncoding.PauliBlockEncoding;
import JordanWigner.JordanWignerOptimizedBlockEncoding.QuantumWalkByQubitization;
import JordanWigner.Utils.JordanWignerEncodingData;
import JordanWigner.Utils.TrotterSimulationAlgorithm;
import Utils.EvolutionGenerator;

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
function TrotterStepOracle(jwHamiltonian : JordanWignerEncodingData, trotterStepSize : Double, trotterOrder : Int) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let generatorSystem = JordanWignerGeneratorSystem(jwHamiltonian.Terms);
    let evolutionGenerator = new EvolutionGenerator { EvolutionSet = JordanWignerFermionEvolutionSet(), System = generatorSystem };
    let simulationAlgorithm = TrotterSimulationAlgorithm(trotterStepSize, trotterOrder);
    let oracle = simulationAlgorithm(trotterStepSize, evolutionGenerator, _);
    let nTargetRegisterQubits = jwHamiltonian.NumQubits;
    let rescaleFactor = 1.0 / trotterStepSize;
    return (nTargetRegisterQubits, (rescaleFactor, oracle));
}


function QubitizationOracleSeperatedRegisters(jwHamiltonian : JordanWignerEncodingData) : ((Int, Int), (Double, ((Qubit[], Qubit[]) => Unit is Adj + Ctl))) {
    let generatorSystem = JordanWignerBlockEncodingGeneratorSystem(jwHamiltonian.Terms);
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
function QubitizationOracle(jwHamiltonian : JordanWignerEncodingData) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, oracle)) = QubitizationOracleSeperatedRegisters(jwHamiltonian);
    let nQubits = nCtrlRegisterQubits + nTargetRegisterQubits;
    return (nQubits, (oneNorm, MergeTwoRegisters(oracle, nTargetRegisterQubits, _)));
}


operation MergeTwoRegisters(oracle : ((Qubit[], Qubit[]) => Unit is Adj + Ctl), nSystemQubits : Int, allQubits : Qubit[]) : Unit is Adj + Ctl {
    oracle(allQubits[nSystemQubits..Length(allQubits) - 1], allQubits[0..nSystemQubits - 1]);
}


function OptimizedQubitizationOracleSeperatedRegisters(jwHamiltonian : JordanWignerEncodingData, targetError : Double) : ((Int, Int), (Double, ((Qubit[], Qubit[]) => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, blockEncodingReflection)) = JordanWignerOptimizedBlockEncoding(targetError, jwHamiltonian.Terms, jwHamiltonian.NumQubits);
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
function OptimizedQubitizationOracle(jwHamiltonian : JordanWignerEncodingData, targetError : Double) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, oracle)) = OptimizedQubitizationOracleSeperatedRegisters(jwHamiltonian, targetError);
    let nQubits = nCtrlRegisterQubits + nTargetRegisterQubits;
    return (nQubits, (oneNorm, MergeTwoRegisters(oracle, nTargetRegisterQubits, _)));
}

// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export
    TrotterStepOracle,
    QubitizationOracle,
    OptimizedQubitizationOracle;

import JordanWigner.JordanWignerBlockEncoding.JordanWignerBlockEncodingGeneratorSystem;
import JordanWigner.JordanWignerEvolutionSet.JordanWignerFermionEvolutionSet;
import JordanWigner.JordanWignerEvolutionSet.JordanWignerGeneratorSystem;
import JordanWigner.JordanWignerOptimizedBlockEncoding.JordanWignerOptimizedBlockEncoding;
import JordanWigner.JordanWignerOptimizedBlockEncoding.PauliBlockEncoding;
import JordanWigner.JordanWignerOptimizedBlockEncoding.QuantumWalkByQubitization;
import JordanWigner.StatePreparation.PrepareArbitraryStateD;
import JordanWigner.Utils.JordanWignerEncodingData;
import JordanWigner.Utils.MultiplexOperationsFromGenerator;
import JordanWigner.Utils.TrotterSimulationAlgorithm;
import Std.Convert.IntAsDouble;
import Std.Math.Ceiling;
import Std.Math.Lg;
import Utils.EvolutionGenerator;
import Utils.GeneratorSystem;

// Convenience functions for performing simulation.

/// # Summary
/// Returns Trotter step operation and the parameters necessary to run it.
///
/// # Input
/// ## qSharpData
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
function TrotterStepOracle(qSharpData : JordanWignerEncodingData, trotterStepSize : Double, trotterOrder : Int) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let (nSpinOrbitals, data, statePrepData, energyShift) = qSharpData!;
    let generatorSystem = JordanWignerGeneratorSystem(data);
    let evolutionGenerator = new EvolutionGenerator { EvolutionSet = JordanWignerFermionEvolutionSet(), System = generatorSystem };
    let simulationAlgorithm = TrotterSimulationAlgorithm(trotterStepSize, trotterOrder);
    let oracle = simulationAlgorithm(trotterStepSize, evolutionGenerator, _);
    let nTargetRegisterQubits = nSpinOrbitals;
    let rescaleFactor = 1.0 / trotterStepSize;
    return (nTargetRegisterQubits, (rescaleFactor, oracle));
}


function QubitizationOracleSeperatedRegisters(qSharpData : JordanWignerEncodingData) : ((Int, Int), (Double, ((Qubit[], Qubit[]) => Unit is Adj + Ctl))) {
    let (nSpinOrbitals, data, statePrepData, energyShift) = qSharpData!;
    let generatorSystem = JordanWignerBlockEncodingGeneratorSystem(data);
    let (nTerms, genIdxFunction) = generatorSystem!;
    let (oneNorm, blockEncodingReflection) = PauliBlockEncoding(generatorSystem);
    let nTargetRegisterQubits = nSpinOrbitals;
    let nCtrlRegisterQubits = Ceiling(Lg(IntAsDouble(nTerms)));
    return ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, QuantumWalkByQubitization(blockEncodingReflection)));
}

/// # Summary
/// Returns Qubitization operation and the parameters necessary to run it.
///
/// # Input
/// ## qSharpData
/// Hamiltonian described by `JordanWignerEncodingData` format.
///
/// # Output
/// A tuple where: `Int` is the number of qubits allocated,
/// `Double` is the one-norm of Hamiltonian coefficients, and the operation
/// is the Quantum walk created by Qubitization.
function QubitizationOracle(qSharpData : JordanWignerEncodingData) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, oracle)) = QubitizationOracleSeperatedRegisters(qSharpData);
    let nQubits = nCtrlRegisterQubits + nTargetRegisterQubits;
    return (nQubits, (oneNorm, MergeTwoRegisters(oracle, nTargetRegisterQubits, _)));
}


operation MergeTwoRegisters(oracle : ((Qubit[], Qubit[]) => Unit is Adj + Ctl), nSystemQubits : Int, allQubits : Qubit[]) : Unit is Adj + Ctl {
    oracle(allQubits[nSystemQubits..Length(allQubits) - 1], allQubits[0..nSystemQubits - 1]);
}


function OptimizedQubitizationOracleSeperatedRegisters(qSharpData : JordanWignerEncodingData, targetError : Double) : ((Int, Int), (Double, ((Qubit[], Qubit[]) => Unit is Adj + Ctl))) {
    let (nSpinOrbitals, data, statePrepData, energyShift) = qSharpData!;
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, blockEncodingReflection)) = JordanWignerOptimizedBlockEncoding(targetError, data, nSpinOrbitals);
    return ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, QuantumWalkByQubitization(blockEncodingReflection)));
}


/// # Summary
/// Returns T-count optimized Qubitization operation
/// and the parameters necessary to run it.
///
/// # Input
/// ## qSharpData
/// Hamiltonian described by `JordanWignerEncodingData` format.
/// ## targetError
/// Error of auxillary state preparation step.
///
/// # Output
/// A tuple where: `Int` is the number of qubits allocated,
/// `Double` is the one-norm of Hamiltonian coefficients, and the operation
/// is the Quantum walk created by Qubitization.
function OptimizedQubitizationOracle(qSharpData : JordanWignerEncodingData, targetError : Double) : (Int, (Double, (Qubit[] => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, oracle)) = OptimizedQubitizationOracleSeperatedRegisters(qSharpData, targetError);
    let nQubits = nCtrlRegisterQubits + nTargetRegisterQubits;
    return (nQubits, (oneNorm, MergeTwoRegisters(oracle, nTargetRegisterQubits, _)));
}

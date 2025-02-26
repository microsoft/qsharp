// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export OptimizedBEXY;

import Std.Arrays.IndexRange;
import Std.Arrays.Partitioned;
import Std.Math.Max;

import JordanWigner.Utils.MultiplexOperationsFromGenerator;

/// # Summary
/// Applies a sequence of Z operations and either an X or Y operation to
/// a register of qubits, where the selection of target qubits and basis
/// are conditioned on the state of a control register.
///
/// # Description
/// This operation can be described by a unitary matrix $U$ that applies
/// the Pauli string on $(X^{z+1}\_pY^{z}\_p)Z\_{p-1}...Z_0$ on
/// qubits $0..p$ conditioned on an index $z\in\{0,1\}$ and $p$.
///
/// That is,
/// $$
/// \begin{align}
/// U\ket{z}\ket{p}\ket{\psi} = \ket{z}\ket{p}(X^{z+1}\_pY^{z}\_p)Z\_{p-1}...Z_0\ket{\psi}
/// \end{align}
/// $$
///
/// # Input
/// ## pauliBasis
/// When this qubit is in state $\ket{0}$, an `X` operation is applied. When it is in state $\ket{1}$, `Y` is applied.
/// ## indexRegister
/// The state $\ket{p}$ of this register determines the qubit on which `X` or `Y` is applied.
/// ## targetRegister
/// Register of qubits on which the Pauli operators are applied.
///
/// # References
/// - [Encoding Electronic Spectra in Quantum Circuits with Linear T Complexity](https://arxiv.org/abs/1805.03662)
///   Ryan Babbush, Craig Gidney, Dominic W. Berry, Nathan Wiebe, Jarrod McClean, Alexandru Paler, Austin Fowler, Hartmut Neven
operation OptimizedBEXY(pauliBasis : Qubit, indexRegister : Qubit[], targetRegister : Qubit[]) : Unit is Adj + Ctl {
    let unitaryGenerator = (Length(targetRegister), idx -> OptimizedBEXYImpl(idx, _, _, _));

    use accumulator = Qubit();

    // This assumes that MultiplexOperationsFromGenerator applies unitaries indexed in unitaryGenerator in ascending order.
    X(accumulator);
    MultiplexOperationsFromGenerator(unitaryGenerator, indexRegister, (pauliBasis, accumulator, targetRegister));
    // If indexRegister encodes an integer that is larger than Length(targetRegister),
    // _OptimizedBEXY_ will fail due to an out of range error. In this situation,
    // releasing the accumulator qubit will throw an error as it will be in the One state.
}

// Subroutine of OptimizedBEXY.
operation OptimizedBEXYImpl(targetIndex : Int, pauliBasis : Qubit, accumulator : Qubit, targetRegister : Qubit[]) : Unit is Adj + Ctl {

    body (...) {
        // This should always be called as a controlled operation.
        fail "_OptimizedBEXY should always be called as a controlled operation.";
    }

    controlled (ctrl, ...) {
        if Length(targetRegister) <= targetIndex {
            fail "targetIndex out of range.";
        }

        Controlled X(ctrl, accumulator);
        within {
            Controlled Adjoint S([pauliBasis], targetRegister[targetIndex]);
        } apply {
            Controlled X(ctrl, targetRegister[targetIndex]);
        }
        Controlled Z([accumulator], targetRegister[targetIndex]);
    }

}

/// # Summary
/// Applies a Z operation to a qubit indicated by the state of another
/// register.
///
/// # Description
/// The operation can be represented by a unitary matrix $U$ that applies
/// the `Std.Intrinsic.Z` operation on a qubit $p$
/// conditioned on an index state $\ket{p}$. That is,
/// $$
/// \begin{align}
///     U\ket{p}\ket{\psi} = \ket{p}Z\_p\ket{\psi}.
/// \end{align}
/// $$
///
/// # Input
/// ## indexRegister
/// A register in the state $\ket{p}$, determining the qubit on which $Z$ is applied.
/// ## targetRegister
/// Register of qubits on which the Pauli operators are applied.
operation SelectZ(indexRegister : Qubit[], targetRegister : Qubit[]) : Unit is Adj + Ctl {
    let unitaryGenerator = (Length(targetRegister), idx -> (qs => Z(qs[idx])));
    MultiplexOperationsFromGenerator(unitaryGenerator, indexRegister, targetRegister);
    // If indexRegister encodes an integer that is larger than Length(targetRegister),
    // _SelectZ_ will fail due to an out of range error. In this situation,
    // releasing the accumulator qubit will throw an error as it will be in the One state.
}

operation JordanWignerSelect(
    signQubit : Qubit,
    selectZControlRegisters : Qubit[],
    OptimizedBEControlRegisters : Qubit[],
    pauliBases : Qubit[],
    indexRegisters : Qubit[][],
    targetRegister : Qubit[]
) : Unit is Adj + Ctl {
    Z(signQubit);

    for idxRegister in IndexRange(OptimizedBEControlRegisters) {
        Controlled OptimizedBEXY([OptimizedBEControlRegisters[idxRegister]], (pauliBases[idxRegister], indexRegisters[idxRegister], targetRegister));
    }

    for idxRegister in IndexRange(selectZControlRegisters) {
        Controlled SelectZ([selectZControlRegisters[idxRegister]], (indexRegisters[idxRegister], targetRegister));
    }
}

function JordanWignerSelectQubitCount(nZ : Int, nMaj : Int, nIdxRegQubits : Int) : (Int, (Int, Int, Int, Int, Int[])) {
    let signQubit = 1;
    let selectZControlRegisters = nZ;
    let OptimizedBEControlRegisters = nMaj;
    let pauliBases = nMaj;
    let indexRegisters = Repeated(nIdxRegQubits, Max([nZ, nMaj]));
    let nTotal = ((1 + nZ) + 2 * nMaj) + Max([nZ, nMaj]) * nIdxRegQubits;
    return (nTotal, (signQubit, selectZControlRegisters, OptimizedBEControlRegisters, pauliBases, indexRegisters));
}

function JordanWignerSelectQubitManager(nZ : Int, nMaj : Int, nIdxRegQubits : Int, ctrlRegister : Qubit[], targetRegister : Qubit[]) : ((Qubit, Qubit[], Qubit[], Qubit[], Qubit[][], Qubit[]), Qubit[]) {
    let (nTotal, (a, b, c, d, e)) = JordanWignerSelectQubitCount(nZ, nMaj, nIdxRegQubits);
    let split = [a, b, c, d] + e;
    let registers = Partitioned(split, ctrlRegister);
    let signQubit = registers[0];
    let selectZControlRegisters = registers[1];
    let OptimizedBEControlRegisters = registers[2];
    let pauliBases = registers[3];
    let indexRegistersTmp = registers[4..(4 + Length(e)) - 1];
    let rest = registers[Length(registers) - 1];
    mutable indexRegisters = [];

    for idx in IndexRange(e) {
        indexRegisters += [indexRegistersTmp[idx]];
    }

    return ((signQubit[0], selectZControlRegisters, OptimizedBEControlRegisters, pauliBases, indexRegisters, targetRegister), rest);
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// TODO: This should be generated from the /samples directory, not hard-coded.

const minimal = `namespace Sample {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation Main() : Result[] {
        // TODO
        return [];
    }
}`;

const bellState = `namespace Sample {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation Main() : Result[] {
        use q1 = Qubit();
        use q2 = Qubit();

        H(q1);
        CNOT(q1, q2);
        DumpMachine();

        let m1 = M(q1);
        let m2 = M(q2);

        Reset(q1);
        Reset(q2);

        return [m1, m2];
    }
}`;

const teleportation = `namespace Microsoft.Quantum.Samples.Teleportation {
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Random; 

    //////////////////////////////////////////////////////////////////////////
    // Introduction //////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////////

    // Quantum teleportation provides a way of moving a quantum state from one
    // location to another without having to move physical particle(s) along
    // with it. This is done with the help of previously shared quantum
    // entanglement between the sending and the receiving locations and
    // classical communication.

    //////////////////////////////////////////////////////////////////////////
    // Teleportation /////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////////

    /// # Summary
    /// Sends the state of one qubit to a target qubit by using
    /// teleportation.
    ///
    /// Notice that after calling Teleport, the state of "msg" is
    /// collapsed.
    ///
    /// # Input
    /// ## msg
    /// A qubit whose state we wish to send.
    /// ## target
    /// A qubit initially in the |0〉 state that we want to send
    /// the state of msg to.
    operation Teleport (msg : Qubit, target : Qubit) : Unit {
        use register = Qubit();
        // Create some entanglement that we can use to send our message.
        H(register);
        CNOT(register, target);

        // Encode the message into the entangled pair.
        CNOT(msg, register);
        H(msg);

        // Measure the qubits to extract the classical data we need to
        // decode the message by applying the corrections on
        // the target qubit accordingly.
        if M(msg) == One { Z(target); }
        // Correction step
        if M(register) == One {
            X(target);
            // Reset register to Zero state before releasing
            X(register);
        }
    }

    // One can use quantum teleportation circuit to send an unobserved
    // (unknown) classical message from source qubit to target qubit
    // by sending specific (known) classical information from source
    // to target.

    /// # Summary
    /// Uses teleportation to send a classical message from one qubit
    /// to another.
    ///
    /// # Input
    /// ## message
    /// If \`true\`, the source qubit (\`here\`) is prepared in the
    /// |1〉 state, otherwise the source qubit is prepared in |0〉.
    ///
    /// ## Output
    /// The result of a Z-basis measurement on the teleported qubit,
    /// represented as a Bool.
    operation TeleportClassicalMessage (message : Bool) : Bool {
        // Ask for some qubits that we can use to teleport.
        use (msg, target) = (Qubit(), Qubit());

        // Encode the message we want to send.
        if message {
            X(msg);
        }

        // Use the operation we defined above.
        Teleport(msg, target);

        // Check what message was received.
        let result = (M(target) == One);
        
        // Reset qubits to Zero state before releasing
        Reset(msg);
        Reset(target);

        return result;
    }

    /// # Summary
    /// Sets the qubit's state to |+⟩.
    operation SetToPlus(q: Qubit) : Unit {
        Reset(q);
        H(q);
    }

    /// # Summary
    /// Sets the qubit's state to |−⟩.
    operation SetToMinus(q: Qubit) : Unit {
        Reset(q);
        X(q);
        H(q);
    }

    /// # Summary
    /// Returns true if qubit is |+⟩ (assumes qubit is either |+⟩ or |−⟩)
    operation MeasureIsPlus(q: Qubit) : Bool {
        Measure([PauliX], [q]) == Zero
    }

    /// # Summary
    /// Returns true if qubit is |−⟩ (assumes qubit is either |+> or |−⟩)
    operation MeasureIsMinus(q: Qubit) : Bool {
        Measure([PauliX], [q]) == One
    }

    /// # Summary
    /// Randomly prepares the qubit into |+⟩ or |−⟩
    operation PrepareRandomMessage(q: Qubit) : Unit {        
        let choice = DrawRandomInt(0, 1) == 1;

        if choice {
            Message("Sending |->");
            SetToMinus(q);
        } else {
            Message("Sending |+>");
            SetToPlus(q);
        }
    }

    // One can also use quantum teleportation to send any quantum state
    // without losing any information. The following sample shows
    // how a randomly picked non-trivial state (|-> or |+>)
    // gets moved from one qubit to another.

    /// # Summary
    /// Uses teleportation to send a randomly picked |-> or |+> state
    /// to another.
    operation TeleportRandomMessage () : Unit {
        // Ask for some qubits that we can use to teleport.
        use (msg, target) = (Qubit(), Qubit());
        PrepareRandomMessage(msg);

        // Use the operation we defined above.
        Teleport(msg, target);

        // Report message received:
        if MeasureIsPlus(target) { Message("Received |+>"); }
        if MeasureIsMinus(target) { Message("Received |->"); }

        // Reset all of the qubits that we used before releasing
        // them.
        Reset(msg);
        Reset(target);
    }

    @EntryPoint()
    operation Main () : Unit {
        for idxRun in 1 .. 10 {
            let sent = DrawRandomInt(0, 1) == 1;
            let received = TeleportClassicalMessage(sent);
            Message($"Round {idxRun}: Sent {sent}, got {received}.");
            Message(sent == received ? "Teleportation successful!" | "");
        }
        for idxRun in 1 .. 10 {
            TeleportRandomMessage();
        }

    }
}

// ////////////////////////////////////////////////////////////////////////
// Other teleportation scenarios not illustrated here
// ////////////////////////////////////////////////////////////////////////

// ● Teleport a rotation. Rotate a basis state by a certain angle φ ∈ [0, 2π),
// for example by preparing Rₓ(φ) |0〉, and teleport the rotated state to the target qubit.
// When successful, the target qubit captures the angle φ [although, on course one does
// not have classical access to its value].
// ● "Super dense coding".  Given an EPR state |β〉 shared between the source and target
// qubits, the source can encode two classical bits a,b by applying Z^b X^a to its half
// of |β〉. Both bits can be recovered on the target by measurement in the Bell basis.
// For details refer to discussion and code in Unit Testing Sample, in file SuperdenseCoding.qs.
// ////////////////////////////////////////////////////////////////////////`;

const qrng = `namespace Microsoft.Quantum.Samples.Qrng {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Diagnostics;

    operation SampleQuantumRandomNumberGenerator() : Result {
        use q = Qubit();   // Allocate a qubit.
        H(q);              // Put the qubit to superposition. It now has a 50% chance of being 0 or 1.
        let result = M(q); // Measure the qubit value, but don't look at the result yet.
        Reset(q);          // Reset qubit to Zero state.
        return result;     // Return result of the measurement.
    }

    operation SampleRandomNumberInRange(max : Int) : Int {
        mutable bits = [];
        for idxBit in 1..BitSizeI(max) {
            set bits += [SampleQuantumRandomNumberGenerator()];
        }
        let sample = ResultArrayAsInt(bits);
        return sample > max
               ? SampleRandomNumberInRange(max)
               | sample;
    }

    /// Produces a non-negative integer from a string of bits in little endian format.
    function ResultArrayAsInt(input : Result[]) : Int {
        let nBits = Length(input);
        // We are constructing a 64-bit integer, and we won't use the highest (sign) bit.
        Fact(nBits < 64, "Input length must be less than 64.");
        mutable number = 0;
        for i in 0..nBits-1 {
            if input[i] == One {
                // If we assume loop unrolling, 2^i will be optimized to a constant.
                set number |||= 2^i;
            }
        }
        return number;
    }

    @EntryPoint()
    operation Main() : Int {
        let max = 50;
        Message($"Sampling a random number between 0 and {max}: ");
        return SampleRandomNumberInRange(max);
    }
}`;

const deutsch = `// First, note that every Q# function must have a namespace. We define
// a new one for this purpose.
namespace Microsoft.Quantum.Samples.DeutschJozsa {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;


    //////////////////////////////////////////////////////////////////////////
    // Deutsch–Jozsa Quantum Algorithm ///////////////////////////////////////
    //////////////////////////////////////////////////////////////////////////

    /// # Summary
    /// Deutsch–Jozsa is a quantum algorithm that decides whether a given Boolean function
    /// 𝑓 that is promised to either be constant or to be balanced — i.e., taking the
    /// values 0 and 1 the exact same number of times — is actually constant or balanced.
    /// The operation \`IsConstantBooleanFunction\` answers this question by returning the
    /// Boolean value \`true\` if the function is constant and \`false\` if it is not. Note
    /// that the promise that the function is either constant or balanced is assumed.
    ///
    /// # Input
    /// ## Uf
    /// A quantum operation that implements |𝑥〉|𝑦〉 ↦ |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉,
    /// where 𝑓 is a Boolean function, 𝑥 is an 𝑛 bit register and 𝑦 is a single qubit.
    /// ## n
    /// The number of bits of the input register |𝑥〉.
    ///
    /// # Output
    /// A boolean value \`true\` that indicates that the function is constant and \`false\`
    /// that indicates that the function is balanced.
    ///
    /// # See Also
    /// - For details see Section 1.4.3 of Nielsen & Chuang.
    ///
    /// # References
    /// - [ *Michael A. Nielsen , Isaac L. Chuang*,
    ///     Quantum Computation and Quantum Information ](http://doi.org/10.1017/CBO9780511976667)
    operation IsConstantBooleanFunction (Uf : ((Qubit[], Qubit) => Unit), n : Int) : Bool {
        // Now, we allocate n + 1 clean qubits. Note that the function Uf is defined
        // on inputs of the form (x, y), where x has n bits and y has 1 bit.
        use queryRegister = Qubit[n];
        use target = Qubit();
        // The last qubit needs to be flipped so that the function will
        // actually be computed into the phase when Uf is applied.
        X(target);

        // Now, a Hadamard transform is applied to each of the qubits.

        H(target);
        // We use a within-apply block to ensure that the Hadamard transform is
        // correctly inverted on |𝑥〉 register.
        within {
            for q in queryRegister {
                H(q);
            }
        } apply {
            // We now apply Uf to the n + 1 qubits, computing |𝑥, 𝑦〉 ↦ |𝑥, 𝑦 ⊕ 𝑓(𝑥)〉.
            Uf(queryRegister, target);
        }

        // The following for-loop measures all qubits and resets them to
        // zero so that they can be safely returned at the end of the using-block.
        // The loop also leaves result as \`true\` if all measurement results
        // are \`Zero\`, i.e., if the function was a constant function and sets
        // result to \`false\` if not, which according to the promise on 𝑓 means
        // that it must have been balanced.
        mutable result = true;
        for q in queryRegister {
            if M(q) == One {
                X(q);
                set result = false;
            }
        }

        // Finally, the last qubit, which held the 𝑦-register, is reset.
        Reset(target);

        return result;
    }

    // Simple constant Boolean function
    operation SimpleConstantBooleanFunction(args: Qubit[], target: Qubit): Unit {
        X(target);
    }

    // A more complex constant Boolean function. It applies X for every input basis vector.
    operation ConstantBooleanFunction(args: Qubit[], target: Qubit): Unit {
        for i in 0..(2^Length(args))-1 {
            ApplyControlledOnInt(i, args, X, target);
        }
    }

    // A more complex balanced Boolean function. It applies X for half of the input basis verctors.
    operation BalancedBooleanFunction(args: Qubit[], target: Qubit): Unit {
        for i in 0..2..(2^Length(args))-1 {
            ApplyControlledOnInt(i, args, X, target);
        }
    }

    // Applies operator \`op\` on each qubit in the \`qubits\` array if the corresponding
    // bit in the LittleEndian \`number\` matches the given \`bitApply\`.
    operation ApplyOpFromInt(
        number: Int,
        bitApply: Bool,
        op:(Qubit => Unit is Adj),
        qubits: Qubit[]): Unit is Adj {

        Fact(number>=0, "number must be non-negative");

        for i in 0..qubits::Length-1 {
            // If we assume loop unrolling, 2^i will be optimized to a constant.
            if (((number &&& 2^i) != 0) == bitApply) {
                op(qubits[i]);
            }
        }
    }

    // Applies a unitary operation \`oracle\` on the target qubit if the control
    // register state corresponds to a specified nonnegative integer \`numberState\`.
    operation ApplyControlledOnInt(
        numberState: Int,
        controls: Qubit[],
        oracle:(Qubit => Unit is Ctl),
        target: Qubit): Unit {

        within {
            ApplyOpFromInt(numberState, false, X, controls);
        } apply {
            Controlled oracle(controls, target);
        }
    }

    @EntryPoint()
    operation Main() : Unit {
        // Constant versus Balanced Functions with the Deutsch–Jozsa Algorithm:

        // A Boolean function is a function that maps bitstrings to a
        // bit,
        //
        //     𝑓 : {0, 1}^n → {0, 1}.
        //
        // We say that 𝑓 is constant if 𝑓(𝑥⃗) = 𝑓(𝑦⃗) for all bitstrings
        // 𝑥⃗ and 𝑦⃗, and that 𝑓 is balanced if 𝑓 evaluates to true (1) for
        // exactly half of its inputs.

        // If we are given a function 𝑓 as a quantum operation 𝑈 |𝑥〉|𝑦〉
        // = |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉, and are promised that 𝑓 is either constant or
        // is balanced, then the Deutsch–Jozsa algorithm decides between
        // these cases with a single application of 𝑈.

        // In SimpleAlgorithms.qs, we implement this algorithm as
        // RunDeutschJozsa, following the pattern above.
        // We check by ensuring that RunDeutschJozsa returns true
        // for constant functions and false for balanced functions.

        if (not IsConstantBooleanFunction(SimpleConstantBooleanFunction, 5)) {
            fail "SimpleConstantBooleanFunction should be detected as constant";
        }
        Message("SimpleConstantBooleanFunction detected as constant");

        if (not IsConstantBooleanFunction(ConstantBooleanFunction, 5)) {
            fail "ConstantBooleanFunction should be detected as constant";
        }
        Message("ConstantBooleanFunction detected as constant");

        if (IsConstantBooleanFunction(BalancedBooleanFunction, 5)) {
            fail "BalancedBooleanFunction should be detected as balanced";
        }
        Message("BalancedBooleanFunction detected as balanced");

        Message("All functions measured successfully!");
    }
}`;

const grover = `// This is NOT Gover's, but does show phases nicely.

namespace Sample {
    @EntryPoint()

    operation AllBasisVectorsWithPhases_TwoQubits() : Unit {
        use q1 = Qubit();
        use q4 = Qubit();

        H(q1);
        R1(0.3, q1);
        H(q4);

        use q5 = Qubit();
        use q6 = Qubit();
        S(q5);

        Rxx(1.0, q5, q6);

        Microsoft.Quantum.Diagnostics.DumpMachine();
    }
}`;

export const samples = {
  Minimal: minimal,
  "Bell state": bellState,
  Teleportation: teleportation,
  "Random numbers": qrng,
  "Deutsch-Josza": deutsch,
  "Grover's search": grover,
  "Shor's algorithm": "// TODO",
};

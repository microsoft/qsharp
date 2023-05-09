// First, note that every Q# function must have a namespace. We define
// a new one for this purpose.
namespace Microsoft.Quantum.Samples.DeutschJozsa {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;


    ////////////////////////////////////////////////////////////////////
    // Deutsch–Jozsa Quantum Algorithm /////////////////////////////////
    ////////////////////////////////////////////////////////////////////

    /// # Summary
    /// Deutsch–Jozsa is a quantum algorithm that decides whether a
    /// given Boolean function 𝑓 that is promised to either be constant
    /// or to be balanced — i.e., taking the values 0 and 1 the exact
    /// same number of times — is actually constant or balanced.
    /// The operation `IsConstantBooleanFunction` answers this question
    /// by returning the Boolean value `true` if the function is
    /// constant and `false` if it is not. Note that the promise that
    /// the function is either constant or balanced is assumed.
    ///
    /// # Input
    /// ## Uf
    /// A quantum operation that implements |𝑥〉|𝑦〉 ↦ |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉,
    /// where 𝑓 is a Boolean function, 𝑥 is an 𝑛 bit register and 𝑦 is
    /// a single qubit.
    /// ## n
    /// The number of bits of the input register |𝑥〉.
    ///
    /// # Output
    /// A boolean value `true` that indicates that the function is
    /// constant and `false`
    /// that indicates that the function is balanced.
    ///
    /// # See Also
    /// - For details see Section 1.4.3 of Nielsen & Chuang.
    ///
    /// # References
    /// - [ *Michael A. Nielsen , Isaac L. Chuang*,
    ///     Quantum Computation and Quantum Information ]
    /// (http://doi.org/10.1017/CBO9780511976667)
    operation IsConstantBooleanFunction(
        Uf: ((Qubit[], Qubit) => Unit),
        n: Int): Bool {

        // Now, we allocate n + 1 clean qubits. Note that the function
        // Uf is defined on inputs of the form (x, y), where x has n
        // bits and y has 1 bit.
        use queryRegister = Qubit[n];
        use target = Qubit();
        // The last qubit needs to be flipped so that the function will
        // actually be computed into the phase when Uf is applied.
        X(target);

        // Now, a Hadamard transform is applied to each of the qubits.

        H(target);
        // We use a within-apply block to ensure that the Hadamard
        // transform is correctly inverted on |𝑥〉 register.
        within {
            for q in queryRegister {
                H(q);
            }
        } apply {
            // We now apply Uf to the n + 1 qubits, computing
            // |𝑥, 𝑦〉 ↦ |𝑥, 𝑦 ⊕ 𝑓(𝑥)〉.
            Uf(queryRegister, target);
        }

        // The following for-loop measures all qubits and resets them to
        // zero so that they can be safely returned at the end of the
        // using-block. The loop also leaves result as `true` if all
        // measurement results are `Zero`, i.e., if the function was a
        // constant function and sets result to `false` if not, which
        // according to the promise on 𝑓 means that it must have been
        // balanced.
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
    operation SimpleConstantBoolF(
        args: Qubit[],
        target: Qubit): Unit {

        X(target);
    }

    // Simple balanced Boolean function
    operation SimpleBalancedBoolF(
        args: Qubit[],
        target: Qubit): Unit {

        CX(args[0], target);
    }

    // A more complex constant Boolean function.
    // It applies X for every input basis vector.
    operation ConstantBoolF(
        args: Qubit[],
        target: Qubit): Unit {

        for i in 0..(2^args::Length)-1 {
            ApplyControlledOnInt(i, args, X, target);
        }
    }

    // A more complex balanced Boolean function. It applies X for
    // half of the input basis verctors.
    operation BalancedBoolF(
        args: Qubit[], target: Qubit): Unit {

        for i in 0..2..(2^args::Length)-1 {
            ApplyControlledOnInt(i, args, X, target);
        }
    }

    // Applies operator `op` on each qubit in the `qubits` array if the
    // corresponding bit in the LittleEndian `number` matches the given
    // `bitApply`.
    operation ApplyOpFromInt(
        number: Int,
        bitApply: Bool,
        op:(Qubit => Unit is Adj),
        qubits: Qubit[]): Unit is Adj {

        Fact(number>=0, "number must be non-negative");

        for i in 0..qubits::Length-1 {
            // If we assume loop unrolling, 2^i
            // will be optimized to a constant.
            if (((number &&& 2^i) != 0) == bitApply) {
                op(qubits[i]);
            }
        }
    }

    // Applies a unitary operation `oracle` on the target qubit
    // if the control register state corresponds to a specified
    // nonnegative integer `numberState`.
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
        // Constant versus Balanced Functions with the
        // Deutsch–Jozsa Algorithm:

        // A Boolean function is a function that maps bitstrings to a
        // bit,
        //
        //     𝑓 : {0, 1}^n → {0, 1}.
        //
        // We say that 𝑓 is constant if 𝑓(𝑥⃗) = 𝑓(𝑦⃗) for all bitstrings
        // 𝑥⃗ and 𝑦⃗, and that 𝑓 is balanced if 𝑓 evaluates to true (1)
        // for exactly half of its inputs.

        // If we are given a function 𝑓 as a quantum operation 𝑈 |𝑥〉|𝑦〉
        // = |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉, and are promised that 𝑓 is either constant
        // or is balanced, then the Deutsch–Jozsa algorithm decides
        // between these cases with a single application of 𝑈.

        // In SimpleAlgorithms.qs, we implement this algorithm as
        // RunDeutschJozsa, following the pattern above.
        // We check by ensuring that RunDeutschJozsa returns true
        // for constant functions and false for balanced functions.

        if (not IsConstantBooleanFunction(SimpleConstantBoolF, 5)) {
            fail "SimpleConstantBoolF should be detected as constant";
        }
        Message("SimpleConstantBoolF detected as constant");

        if (IsConstantBooleanFunction(SimpleBalancedBoolF, 5)) {
            fail "SimpleBalancedBoolF should be detected as balanced";
        }
        Message("SimpleBalancedBoolF detected as balanced");

        if (not IsConstantBooleanFunction(ConstantBoolF, 5)) {
            fail "ConstantBoolF should be detected as constant";
        }
        Message("ConstantBoolF detected as constant");

        if (IsConstantBooleanFunction(BalancedBoolF, 5)) {
            fail "BalancedBoolF should be detected as balanced";
        }
        Message("BalancedBoolF detected as balanced");

        Message("All functions measured successfully!");
    }

}

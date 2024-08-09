// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;

    /// # Summary
    /// Performs a modular increment of a qubit register by an integer constant.
    ///
    /// # Description
    /// Let us denote `increment` by $a$, `modulus` by $N$ and integer encoded in `target` by $y$.
    /// Then the operation performs the following transformation:
    /// \begin{align}
    ///     \ket{y} \mapsto \ket{(y + a) \operatorname{mod} N}
    /// \end{align}
    /// Integers are encoded in little-endian format.
    ///
    /// # Input
    /// ## increment
    /// Integer increment $a$ to be added to $y$.
    /// ## modulus
    /// Integer $N$ that mods $y + a$.
    /// ## target
    /// Integer $y$ in `LittleEndian` format that `increment` $a$ is added to.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arithmetic.IncrementPhaseByModularInteger
    ///
    /// # Remarks
    /// Assumes that the initial value of target is less than $N$
    /// and that the increment $a$ is less than $N$.
    /// Note that
    /// <xref:Microsoft.Quantum.Arithmetic.IncrementPhaseByModularInteger> implements
    /// the same operation in the `PhaseLittleEndian` basis.
    operation IncrementByModularInteger(increment : Int, modulus : Int, target : LittleEndian)
    : Unit is Adj + Ctl {
        let inner = IncrementPhaseByModularInteger(increment, modulus, _);

        use extraZeroBit = Qubit();
        ApplyPhaseLEOperationOnLECA(inner, LittleEndian(target! + [extraZeroBit]));
    }

    /// # Summary
    /// Performs a modular increment of a qubit register by an integer constant.
    ///
    /// # Description
    /// Let us denote `increment` by $a$, `modulus` by $N$ and integer encoded in `target` by $y$.
    /// Then the operation performs the following transformation:
    /// \begin{align}
    ///     \ket{y} \mapsto \ket{(y + a) \operatorname{mod} N}
    /// \end{align}
    /// Integers are encoded in little-endian format in QFT basis.
    ///
    /// # Input
    /// ## increment
    /// Integer increment $a$ to be added to $y$.
    /// ## modulus
    /// Integer $N$ that mods $y + a$.
    /// ## target
    /// Integer $y$ in phase-encoded little-endian format that `increment` $a$ is added to.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arithmetic.IncrementByModularInteger
    ///
    /// # Remarks
    /// Assumes that `target` has the highest bit set to 0.
    /// Also assumes that the value of target is less than $N$.
    ///
    /// For the circuit diagram and explanation see Figure 5 on [Page 5
    /// of arXiv:quant-ph/0205095v3](https://arxiv.org/pdf/quant-ph/0205095v3.pdf#page=5).
    operation IncrementPhaseByModularInteger(increment : Int, modulus : Int, target : PhaseLittleEndian)
    : Unit is Adj + Ctl {
        body (...) {
            Controlled IncrementPhaseByModularInteger([], (increment, modulus, target));
        }

        controlled (controls, ...) {
            Fact(modulus <= 2 ^ (Length(target!) - 1), $"`multiplier` must be big enough to fit integers modulo `modulus`" + $"with highest bit set to 0");

            if ExtraArithmeticAssertionsEnabled() {
                // assert that the highest bit is zero, by switching to computational basis
                ApplyLEOperationOnPhaseLEA(AssertMostSignificantBit(Zero, _), target);

                // check that the input is less than modulus
                AssertPhaseLessThan(modulus, target);
            }

            // note that controlled version is correct only under the assumption
            // that the value of target is less than modulus
            use lessThanModulusFlag = Qubit();
            let copyMostSignificantBitPhaseLE = ApplyLEOperationOnPhaseLEA(CopyMostSignificantBit(_, lessThanModulusFlag), _);

            // lets track the state of target register through the computation
            Controlled IncrementPhaseByInteger(controls, (increment, target));

            // the state is |x+a⟩ in QFT basis
            Adjoint IncrementPhaseByInteger(modulus, target);

            // the state is |x+a-N⟩ in QFT basis
            copyMostSignificantBitPhaseLE(target);

            // lessThanModulusFlag is set to 1 if x+a < N
            Controlled IncrementPhaseByInteger([lessThanModulusFlag], (modulus, target));

            // the state is |x+a (mod N)⟩ in QFT basis
            // Now let us restore the lessThanModulusFlag qubit back to zero
            Controlled (Adjoint IncrementPhaseByInteger)(controls, (increment, target));
            X(lessThanModulusFlag);
            copyMostSignificantBitPhaseLE(target);
            Controlled IncrementPhaseByInteger(controls, (increment, target));
        }
    }

    /// # Summary
    /// Performs a modular multiply-and-add by integer constants on a qubit register.
    ///
    /// # Description
    /// Implements the map
    /// $$
    /// \begin{align}
    ///     \ket{x} \ket{b} \mapsto \ket{x} \ket{(b + a \cdot x) \operatorname{mod} N}
    /// \end{align}
    /// $$
    /// for a given modulus $N$, constant multiplier $a$, and summand $b$.
    ///
    /// # Input
    /// ## constMultiplier
    /// An integer $a$ by which `multiplicand` is being multiplied.
    /// Must be between 0 and `modulus`-1, inclusive.
    /// ## modulus
    /// The modulus $N$ which addition and multiplication is taken with respect to.
    /// ## multiplicand
    /// A quantum register representing an unsigned integer whose value, multiplied by `constMultiplier`, is to
    /// be added to each basis state label of `summand`. Corresponds to the
    /// register in state $\ket{x}$ above.
    /// ## summand
    /// A quantum register representing an unsigned integer to use as the target
    /// for this operation. Corresponds to the register initially in $\ket{b}$ above.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arithmetic.MultiplyAndAddPhaseByModularInteger
    ///
    /// # Remarks
    /// - For the circuit diagram and explanation see Figure 6 on [Page 7
    ///        of arXiv:quant-ph/0205095v3](https://arxiv.org/pdf/quant-ph/0205095v3.pdf#page=7)
    /// - This operation corresponds to CMULT(a)MOD(N) in
    ///   [arXiv:quant-ph/0205095v3](https://arxiv.org/pdf/quant-ph/0205095v3.pdf)
    operation MultiplyAndAddByModularInteger(constMultiplier : Int, modulus : Int, multiplicand : LittleEndian, summand : LittleEndian)
    : Unit is Adj + Ctl {
        let inner = MultiplyAndAddPhaseByModularInteger(constMultiplier, modulus, multiplicand, _);

        use extraZeroBit = Qubit();
        ApplyPhaseLEOperationOnLECA(inner, LittleEndian(summand! + [extraZeroBit]));
    }

    /// # Summary
    /// The same as MultiplyAndAddByModularInteger, but assumes that the summand encodes
    /// integers in QFT basis.
    ///
    /// # Input
    /// ## constMultiplier
    /// An integer $a$ by which `multiplicand` is being multiplied.
    /// Must be between 0 and `modulus`-1, inclusive.
    /// ## modulus
    /// The modulus $N$ which addition and multiplication is taken with respect to.
    /// ## multiplicand
    /// A quantum register representing an unsigned integer whose value, multiplied by `constMultiplier`, is to
    /// be added to each basis state label of `summand`.
    /// ## phaseSummand
    /// A quantum register representing an unsigned integer to use as the target
    /// for this operation.
    ///
    /// # Remarks
    /// Assumes that `phaseSummand` has the highest bit set to 0.
    /// Also assumes that the value of `phaseSummand` is less than $N$.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Arithmetic.MultiplyAndAddByModularInteger
    operation MultiplyAndAddPhaseByModularInteger(constMultiplier : Int, modulus : Int, multiplicand : LittleEndian, phaseSummand : PhaseLittleEndian)
    : Unit is Adj + Ctl {
        Fact(modulus <= 2 ^ (Length(phaseSummand!) - 1), $"`multiplicand` must be big enough to fit integers modulo `modulus`" + $"with highest bit set to 0");
        Fact(constMultiplier >= 0 and constMultiplier < modulus, $"`constMultiplier` must be between 0 and `modulus`-1");
 
        if (ExtraArithmeticAssertionsEnabled()) {
            // assert that the highest bit is zero, by switching to computational basis
            ApplyLEOperationOnPhaseLECA(AssertMostSignificantBit(Zero, _), phaseSummand);

            // check that the input is less than modulus
            AssertPhaseLessThan(modulus, phaseSummand);
        }

        for i in IndexRange(multiplicand!) {
            let summand = (ExpModI(2, i, modulus) * constMultiplier) % modulus;
            Controlled IncrementPhaseByModularInteger([(multiplicand!)[i]], (summand, modulus, phaseSummand));
        }
    }

    /// # Summary
    /// Performs modular multiplication by an integer constant on a qubit register.
    ///
    /// # Description
    /// Let us denote `modulus` by $N$ and `constMultiplier` by $a$.
    /// Then this operation implements a unitary operation defined by the following map on the
    /// computational basis:
    /// $$
    /// \begin{align}
    ///     \ket{y} \mapsto \ket{(a \cdot y) \operatorname{mod} N}
    /// \end{align}
    /// $$
    /// for all $y$ between $0$ and $N - 1$.
    ///
    /// # Input
    /// ## constMultiplier
    /// Constant by which multiplicand is being multiplied. Must be co-prime to modulus.
    /// ## modulus
    /// The multiplication operation is performed modulo `modulus`.
    /// ## multiplicand
    /// The number being multiplied by a constant.
    /// This is an array of qubits encoding an integer in little-endian format.
    ///
    /// # Remarks
    /// - For the circuit diagram and explanation see Figure 7 on [Page 8
    ///        of arXiv:quant-ph/0205095v3](https://arxiv.org/pdf/quant-ph/0205095v3.pdf#page=8)
    /// - This operation corresponds to Uₐ in
    ///   [arXiv:quant-ph/0205095v3](https://arxiv.org/pdf/quant-ph/0205095v3.pdf)
    operation MultiplyByModularInteger(constMultiplier : Int, modulus : Int, multiplicand : LittleEndian) : Unit is Adj + Ctl {
        // Check the preconditions using Microsoft.Quantum.Canon.EqualityFactB
        Fact((0 <= constMultiplier and constMultiplier < modulus == true), $"`constMultiplier` must be between 0 and `modulus`");
        EqualityFactB(modulus <= 2 ^ Length(multiplicand!), true, $"`multiplicand` must be big enough to fit integers modulo `modulus`");
        EqualityFactB(IsCoprimeI(constMultiplier, modulus), true, $"`constMultiplier` and `modulus` must be co-prime");

        use summand = Qubit[Length(multiplicand!)];
        // recall that newly allocated qubits are all in 0 state
        // and therefore summandLE encodes 0.
        let summandLE = LittleEndian(summand);

        // Let us look at what is the result of operations below assuming
        // multiplicand is in computational basis and encodes x
        // Currently the joint state of multiplicand and summandLE is
        // |x⟩|0⟩
        MultiplyAndAddByModularInteger(constMultiplier, modulus, multiplicand, summandLE);

        // now the joint state is |x⟩|x⋅a(mod N)⟩
        ApplyToEachCA(SWAP, Zipped(summandLE!, multiplicand!));

        // now the joint state is |x⋅a(mod N)⟩|x⟩
        let inverseMod = InverseModI(constMultiplier, modulus);

        // note that the operation below implements the following map:
        // |x⟩|y⟩ ↦ |x⟩|y - a⁻¹⋅x (mod N)⟩
        Adjoint MultiplyAndAddByModularInteger(inverseMod, modulus, multiplicand, summandLE);
        // now the joint state is |x⋅a(mod N)⟩|x - a⁻¹⋅x⋅a (mod N)⟩ = |x⋅a(mod N)⟩|0⟩
    }


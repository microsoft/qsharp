/// # Sample
/// Bernstein-Vazirani algorithm
///
/// # Description
/// The Bernstein-Vazirani algorithm determines a bit string encoded in a
/// function.
///
/// This Q# program implements the Bernstein-Vazirani algorithm.
namespace Sample {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Measurement;

    /// # Summary
    /// `LearnParityViaFourierSampling` implements the Bernstein-Vazirani
    /// quantum algorithm. This algorithm computes for a given Boolean function
    /// that is promised to be a parity 𝑓(𝑥₀, …, 𝑥ₙ₋₁) = Σᵢ 𝑟ᵢ 𝑥ᵢ a result in the
    /// form of a bit vector (𝑟₀, …, 𝑟ₙ₋₁) corresponding to the parity function.
    /// Note that it is promised that the function is actually a parity
    /// function.
    ///
    /// # Input
    /// ## Uf
    /// A quantum operation that implements |𝑥〉|𝑦〉 ↦ |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉,
    /// where 𝑓 is a Boolean function that implements a parity Σᵢ 𝑟ᵢ 𝑥ᵢ.
    /// ## n
    /// The number of bits in the input register |𝑥〉.
    ///
    /// # Output
    /// An array of type `Result[]` that contains the parity 𝑟⃗ = (𝑟₀, …, 𝑟ₙ₋₁).
    ///
    /// # See Also
    /// - For details see Section 1.4.3 of Nielsen & Chuang.
    ///
    /// # References
    /// - [ *Ethan Bernstein and Umesh Vazirani*,
    ///     SIAM J. Comput., 26(5), 1411–1473, 1997 ]
    ///   (https://doi.org/10.1137/S0097539796300921)
    operation LearnParityViaFourierSampling(
        Uf: ((Qubit[], Qubit) => Unit),
        n : Int) : Result[] {

        // We allocate n + 1 clean qubits. Note that the function Uf is defined
        // on inputs of the form (x, y), where x has n bits and y has 1 bit.
        use queryRegister = Qubit[n];
        use target = Qubit();

        // The last qubit needs to be flipped so that the function will actually
        // be computed into the phase when Uf is applied.
        X(target);

        within {
            // Now, a Hadamard transform is applied to each of the qubits. As
            // the last step before the measurement, a Hadamard transform is
            // applied to all qubits except last one. We could apply the
            // transform to the last qubit also, but this would not affect the
            // final outcome.
            // We use a within-apply block to ensure that the Hadamard transform
            // is correctly inverted.
            ApplyToEachA(H, queryRegister);
        } apply {
            H(target);
            // We now apply Uf to the n+1 qubits, computing
            // |x, y〉 ↦ |x, y ⊕ f(x)〉.
            Uf(queryRegister, target);
        }

        // Measure all qubits and reset them to the |0〉 state so that they can
        // be safely deallocated at the end of the block.
        let resultArray = ForEach(MResetZ, queryRegister);

        // Finally, the last qubit, which held the y-register, is reset.
        Reset(target);

        // The result is already contained in resultArray so no further
        // post-processing is necessary.
        return resultArray;
    }

    // TODO: Move these comments to a place where they fit better.
    // To demonstrate the Bernstein–Vazirani algorithm, we define
    // a function which returns black-box operations (Qubit[] => ()) of
    // the form U_f |𝑥〉|𝑦〉 = |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉, as described above.

    // In particular, we define 𝑓 by providing the pattern 𝑟⃗. Thus,
    // we can easily assert that the pattern measured by the
    // Bernstein–Vazirani algorithm matches the pattern we used
    // to define 𝑓.

    /// # Summary
    /// Given an integer pattern that can be represented as a bitstring
    /// 𝑟⃗ = (r₀, …, rₙ₋₁), this operation applies a unitary 𝑈 that acts on 𝑛 + 1
    /// qubits as:
    ///     𝑈 |𝑥〉|𝑦〉 = |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉
    /// Where 𝑓(𝑥) = Σᵢ 𝑥ᵢ 𝑟ᵢ mod 2.
    ///
    /// # Input
    /// ## pattern
    /// The integer pattern that can be represented as a bitstring 𝑟⃗ used to
    /// define the function 𝑓.
    /// ## xRegister
    /// Represents the |𝑥〉 register that 𝑈 acts on.
    /// ## yQubit
    /// Represents the |𝑦〉 qubit that 𝑈 acts on.
    internal operation ApplyParityOperation(
        pattern: Int,
        xRegister: Qubit[],
        yQubit: Qubit) : Unit {

        // `xRegister` muts have enough qubits to represent the pattern.
        let requiredBits = BitSizeI(pattern);
        let availableQubits = Length(xRegister);
        Fact(
            availableQubits >= requiredBits,
            $"Pattern {pattern} requires {requiredBits} bits to be " +
            $"represented but quantum register only has " +
            $"{availableQubits} qubits");

        // Apply the quantum operations that encode the pattern.
        for index in IndexRange(xRegister) {
            if ((pattern &&& 2^index) != 0) {
                CNOT(xRegister[index], yQubit);
            }
        }
    }

    internal operation EncodePatternInParityOperation(pattern: Int) :
        (Qubit[], Qubit) => Unit {
        return ApplyParityOperation(pattern, _, _);
    }

    // For convenience, we provide an operation that converts a result array
    // into an integer.
    operation RunBernsteinVazirani(
        nQubits: Int,
        Uf : ((Qubit[], Qubit) => Unit)): Int {
        let result = LearnParityViaFourierSampling(Uf, nQubits);
        return ResultArrayAsInt(result);
    }

    @EntryPoint()
    operation Main() : Unit {
        let nQubits = 12;
        // TODO: Consider whether these comments belong here.
        // Parity Sampling with the Bernstein–Vazirani Algorithm:

        // Consider a function 𝑓(𝑥⃗) on bitstrings 𝑥⃗ = (𝑥₀, …, 𝑥ₙ₋₁)
        // of the form
        //
        //     𝑓(𝑥⃗) ≔ Σᵢ 𝑥ᵢ 𝑟ᵢ
        //
        // where 𝑟⃗ = (𝑟₀, …, 𝑟ₙ₋₁) is an unknown bitstring that
        // determines the parity of 𝑓.

        // The Bernstein–Vazirani algorithm allows determining 𝑟 given a
        // quantum operation that implements
        //
        //     |𝑥〉|𝑦〉 ↦ |𝑥〉|𝑦 ⊕ 𝑓(𝑥)〉.
        //
        // In SimpleAlgorithms.qs, we implement this algorithm as the
        // operation RunBernsteinVazirani. This operation takes an
        // integer whose bits describe 𝑟, then uses those bits to
        // construct an appropriate operation, and finally measures 𝑟.

        // We call that operation here, ensuring that we always get the
        // same value for 𝑟 that we provided as input.

        let patterns = [238, 3435];
        for pattern in patterns {
            let encodingOperation = EncodePatternInParityOperation(pattern);
            let decodedPattern = RunBernsteinVazirani(
                nQubits, encodingOperation);
            Fact(
                decodedPattern == pattern,
                $"Decoded pattern {decodedPattern}, but expected {pattern}.");

            Message($"Successfully decoded pattern: {decodedPattern}");
        }
    }

}

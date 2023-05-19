namespace Microsoft.Quantum.Samples.HiddenShift {
    open Microsoft.Quantum.Measurement;
    //open Microsoft.Quantum.Arrays;  // ForEach
    //open Microsoft.Quantum.Convert; // ResultArrayAsInt
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arithmetic;


    // We consider a particular family of problems known as hidden
    // shift problems, in which one is given two Boolean functions
    // 𝑓 and 𝑔 with the promise that they satisfy the relation
    //
    //     𝑔(𝑥) = 𝑓(𝑥 ⊕ 𝑠) for all 𝑥,
    //
    // where 𝑠 is a hidden bitstring that we would like to find.

    // Good quantum algorithms exist for several different families of
    // pairs of Boolean functions. In particular, here we consider the
    // case in which both 𝑓 and 𝑔 are bent functions. We say that
    // a Boolean function is bent if it is as far from linear as
    // possible. In particular, bent functions have flat Fourier spectra,
    // such that each Fourier coefficient is equal in absolute value.

    // In this case, the Roetteler algorithm (see References, below)
    // uses black-box oracles for 𝑓^* and 𝑔, where 𝑓^* is the dual bent
    // function to 𝑓 (defined in more detail below), and computes the
    // hidden shift 𝑠 between 𝑓 and 𝑔.

    /// # Summary
    /// Correlation-based algorithm to solve the hidden shift problem
    /// for bent functions. The problem is to identify an unknown
    /// shift 𝑠 of the arguments of two Boolean functions 𝑓 and 𝑔 that
    /// are promised to satisfy the relation 𝑔(𝑥) = 𝑓(𝑥 ⊕ 𝑠) for all 𝑥.
    /// Note that the promise about the functions 𝑓 and 𝑔 to be
    /// bent functions is assumed, i.e., they both have a flat Fourier
    // (Walsh–Hadamard) spectra. Input to this algorithm are
    /// implementations 𝑈_g of the Boolean function 𝑔 and 𝑈_f^*,
    /// an implementation of dual bent function of the function 𝑓.
    /// Both functions are given via phase encoding.
    ///
    /// # Input
    /// ## Ufstar
    /// A quantum operation that implements
    /// $U_f^*: |𝑥〉 ↦ (-1)^{f^*(x)} |𝑥〉$,
    /// where $f^*$ is a Boolean function, 𝑥 is an $n$ bit register
    /// ## Ug
    /// A quantum operation that implements
    /// $U_g:|𝑥〉 ↦ (-1)^{g(x)} |𝑥〉$,
    /// where 𝑔 is a Boolean function that is shifted by unknown
    /// 𝑠 from 𝑓, and 𝑥 is an $n$ bit register.
    /// ## n
    /// The number of bits of the input register |𝑥〉.
    ///
    /// # Output
    /// An array of type `Result[]` which encodes the bit representation
    /// of the hidden shift.
    ///
    /// # References
    /// - [*Martin Roetteler*,
    ///    Proc. SODA 2010, ACM, pp. 448-457, 2010](https://doi.org/10.1137/1.9781611973075.37)
    operation HiddenShiftBentCorrelation (
        Ufstar: (Qubit[] => Unit),
        Ug: (Qubit[] => Unit),
        n: Int): Result[] {
        // now, we allocate n clean qubits. Note that the function
        // Ufstar and Ug are unitary operations on n qubits defined
        // via phase encoding.
        use qubits = Qubit[n];

        // first, a Hadamard transform is applied to each of the qubits.
        ApplyToEach(H, qubits);

        // we now apply the shifted function Ug to the n qubits,
        // computing |x〉 -> (-1)^{g(x)} |x〉.
        Ug(qubits);

        within {
            // A Hadamard transform is applied to each of the n qubits.
            ApplyToEachA(H, qubits);
        } apply {
            // we now apply the dual function of the unshifted function,
            // i.e., Ufstar, to the n qubits, computing
            // |x〉 -> (-1)^{fstar(x)} |x〉.
            Ufstar(qubits);
        }

        // `ForEach` measures the n qubits and resets them to zero
        // so that they can be safely returned at the end of the
        // using-block.
        return ForEach(MResetZ, qubits);
    }


    // We demonstrate this algorithm by defining an operation which
    // implements an oracle for a bent function constructed from the
    // inner product of Boolean functions.

    // In particular, the operation `InnerProductBentFunction`
    // defines the Boolean function IP(x_0, ..., x_{n-1}) which is
    // computed into the phase, i.e., a diagonal operator that maps
    // |x〉 -> (-1)^{IP(x)} |x〉, where x stands for x=(x_0, ..., x_{n-1})
    // and all the x_i are binary. The IP function is defined as
    // IP(y, z) = y_0 z_0 + y_1 z_1 + ... y_{u-1} z_{u-1} where
    // y = (y_0, ..., y_{u-1}) and z =  (z_0, ..., z_{u-1}) are two bit
    // vectors of length u. Notice that the function IP is a Boolean
    // function on n = 2u bits. IP is a special case of a so-called
    // 'bent' function. These are functions for which the Walsh-Hadamard
    // transform is perfectly flat (in absolute value). Because of this
    // flatness, the Walsh-Hadamard spectrum of any bent function
    // defines a +1/-1 function, i.e., gives rise to another Boolean
    // function, called the 'dual bent function'. What is more, for the
    // case of the IP function it can be shown that IP is equal to
    // its own dual bent function, a fact that is exploited in
    // the present test case.

    // Notice that a diagonal operator implementing IP between
    // 2 variables y_0 and z_0 is nothing but the AND function between
    // those variables, i.e., in phase encoding it is computed by a
    // Controlled-Z gate. Extending this to an XOR of the AND of more
    // variables, as required in the definition of the IP function can
    // then be accomplished by applying several Controlled-Z
    // gates between the respective inputs.
    internal operation InnerProductBentFunction(qs : Qubit[]) : Unit {
        Fact(Length(qs) % 2 == 0, "Length of qs must be even.");

        let u = Length(qs) / 2;

        let xs = qs[0 .. u - 1];
        let ys = qs[u...];

        for i in 0..u-1 {
            CZ(xs[i], ys[i]);
        }
    }

    // To instantiate the hidden shift problem we need another function
    // g which is related to IP via g(x) = IP(x + s), i.e., we have to
    // shift the argument of the IP function by a given shift. Notice
    // that the '+' operation here is the Boolean addition, i.e.,
    // a bit-wise operation. Notice further, that in general a diagonal
    // operation |x〉 -> (-1)^{f(x)} can be turned into a shifted
    // version by applying a bit flip to the |x〉 register first, then
    // applying the diagonal operation, and then undoing the bit flips
    // to the |x〉 register. We use this principle to define shifted
    // versions of the IP operation.
    // This is the version where s = 170 (in binary representation)
    internal operation ShiftedInnerProductBentFunction_170(
        qs : Qubit[]) : Unit {

        Fact(Length(qs) % 2 == 0, "Length of qs must be even.");
        let u = Length(qs) / 2;

        within {
            // the following loop flips the bits in shift
            ApplyXorInPlace(170, qs);
        } apply {
            // now we compute the IP function into the phase
            InnerProductBentFunction(qs);
        }
    }

    // This is the version where s = 3510 (in binary representation)
    internal operation ShiftedInnerProductBentFunction_3510(
        qs : Qubit[]) : Unit {

        Fact(Length(qs) % 2 == 0, "Length of qs must be even.");
        let u = Length(qs)/2;

        within {
            // the following loop flips the bits in shift
            ApplyXorInPlace(3510, qs);
        } apply {
            // now we compute the IP function into the phase
            InnerProductBentFunction(qs);
        }
    }

    // Run the Hidden Shift algorithm on a given number of qubits,
    // given Ug function and Ufstar = InnerProductBentFunction.
    operation FindHiddenShift (Ug : (Qubit[] => Unit), nQubits : Int) : Int {
        let result = HiddenShiftBentCorrelation(
            InnerProductBentFunction,
            Ug,
            nQubits
        );

        // We then convert result back to an integer.
        return ResultArrayAsInt(result);
    }


    // We consider the case of finding a hidden shift 𝑠
    // between two Boolean functions 𝑓(𝑥) and 𝑔(𝑥) = 𝑓(𝑥 ⊕ 𝑠).
    // This problem can be solved on a quantum computer with one call
    // to each of 𝑓 and 𝑔 in the special case that both functions are
    // bent; that is, that they are as far from linear as possible.

    // Here, we run the test case HiddenShiftBentCorrelationTestCase
    // defined in the matching Q# source code, and ensure that it
    // correctly finds each hidden shift for a family of bent
    // functions defined by the inner product.
    @EntryPoint()
    operation Main(): Unit {
        let nQubits = 12;

        let measuredShift = FindHiddenShift(
            ShiftedInnerProductBentFunction_170, nQubits);
        if (measuredShift != 170) {
            fail $"Measured shift {measuredShift}, but expected 170.";
        }
        Message("Measured hidden shift 170 successfully!");
        

        let measuredShift = FindHiddenShift(
            ShiftedInnerProductBentFunction_3510, nQubits);
        if (measuredShift != 3510) {
            fail $"Measured shift {measuredShift}, but expected 3510.";
        }
        Message("Measured hidden shift 3510 successfully!");
    }

    // TODO: Remove this when library function is implemented.
    operation ForEach<'T, 'U> (action : ('T => 'U), array : 'T[]) : 'U[] {
        let length = Length(array);
        if length == 0 {
            return [];
        }
        let first = action(array[0]);
        mutable retval = [first, size = length];
        for idx in 1..length - 1 {
            set retval w/= idx <- action(array[idx]);
        }
        return retval;
    }

    // TODO: Remove this when library function is implemented.
    function ResultArrayAsInt(results : Result[]) : Int {
        let nBits = Length(results);
        Fact(nBits < 64, $"`Length(bits)` must be less than 64, but was {nBits}.");

        mutable number = 0;
        for i in 0 .. nBits - 1 {
            if (results[i] == One) {
                set number += 2 ^ i;
            }
        }

        return number;        
    }

}

open Microsoft.Quantum.Arrays;
open Microsoft.Quantum.Canon;
open Microsoft.Quantum.Convert;
open Microsoft.Quantum.Diagnostics;
open Microsoft.Quantum.Intrinsic;
open Microsoft.Quantum.Math;
import Types.FixedPoint;
import Convert.DoubleAsFixedPoint, Convert.FixedPointAsBoolArray;
import Init.PrepareFxP;
import Addition.SubtractFxP;

/// # Summary
/// The return type when making a lookup table. This contains the operation that
/// makes the lookup table circuit, as well as all the parameters required to make
/// the two FixedPoint registers that need to be used as inputs and outputs to the
/// operator.
///
/// # Remarks
/// The reason we have this return type structure is so that the operator is similar
/// to the other typical Q# arithmetic function implementations (a larger discussion
/// can had as to whether that can be changed)
newtype FunctionWithLookupTable = (
    IntegerBitsIn : Int,
    FractionalBitsIn : Int,
    IntegerBitsOut : Int,
    FractionalBitsOut : Int,
    Apply : (FixedPoint, FixedPoint) => Unit is Adj
);

/// # Summary
/// This function creates a lookup table operator for the function that you want to approximate, as well as
/// the parameters required to make the two `FixedPoint` registers that need to be used as inputs to the operator.
///
/// # Remarks
/// The operator guarantees that given an input value $x$ and a function $f(x)$,
/// it will compute $\hat{f}(\hat{x})$ where $\hat{f}$ is an approximation of $f$ with a maximum error of epsOut and $\hat{x}$ is an
/// approximation of the input value $\hat{x}$ with a maximum error of `epsIn`. This is useful for most reasonably behaved
/// functions, but note that it computes $\hat{f}(\hat{x})$ and not $\hat{f}(x)$ so if the domain function is very oscillatory and/or
/// has funky derivatives then it may have high errors.
///
/// # Input
/// ## func
/// The Q# arithmetic function that you want to implement with the lookup table
/// ## domain
/// A tuple consisting of the minimum and maximum values of the input values to the function
/// ## epsIn
/// The maximum allowed error of the input value to the computation (i.e. |x'-x|)
/// ## epsOut
/// The maximum allowed error of the output without taking into account the error in input value (i.e. |f'(x')-f(x')|)
///
/// # Example
/// The following code creates a quantum operation based on `ExpD` in the (inclusive) range from `-5.0` to `5.0` with an input error of `1e-3` and an output error of `1e-4`.
///
/// ```qsharp
/// // Create operation from lookup table
/// let domain = (-5.0, 5.0);
/// let epsIn = 1e-3;
/// let epsOut = 1e-4;
///
/// let lookup = ApplyFunctionWithLookupTable(ExpD, domain, epsIn, epsOut);
///
/// // Allocate qubits
/// use input = Qubit[lookup::IntegerBitsIn + lookup::FractionalBitsIn];
/// use output = Qubit[lookup::IntegerBitsOut + lookup::FractionalBitsOut];
///
/// // Represent qubit registers as fixed points
/// let inputFxP = FixedPoint(lookup::IntegerBitsIn, input);
/// let outputFxP = FixedPoint(lookup::IntegerBitsOut, output);
///
/// // Apply operation
/// lookup::Apply(inputFxP, outputFxP);
/// ```
function ApplyFunctionWithLookupTable(func : Double -> Double, domain : (Double, Double), epsIn : Double, epsOut : Double) : FunctionWithLookupTable {

    // First step is to find the number of integer bits (pIn) and fractional bits (qIn) required for the input based on the
    // domain and error tolerance (espIn). To find the value of pIn, we have to check both the
    // lower and upper bound of the domain to see which one requires more bits, then assign the larger one as pIn.
    // To find qIn we compute minimum number of fractional bits required to represent epsIn.
    let (minIn, maxIn) = domain;

    let pLower = BitSizeI(Ceiling(AbsD(minIn)));
    let pUpper = BitSizeI(Ceiling(AbsD(maxIn)));
    let pIn = MaxI(pLower, pUpper) + 1; // The +1 is for the sign bit

    let qIn = Ceiling(Lg(1.0 / epsIn));

    // We have now computed the number of integer and fractional bits required for the input of our lookup table. Next we compute
    // The output number of integer and fractional bits required. For the number of fractional bits (qOut), we can
    // simply look at the minimum number of bits required to represent epsOut
    let qOut = Ceiling(Lg(1.0 / epsOut));


    // For the number of integer bits required for the output, we have to iterate through all the possible values of the function
    // and find the one with the largest absolute value. For that we first create the fixed point approximations of minIn and maxIn
    // given epsIn (using the previously computed pIn and qIn). Then we compute how many different input values (numValues) are there between
    // minIn and maxIn (given our number of input qubits). And finally we evaluate the function at all those values to get the number with
    // the largest absolute value

    // Compute approximations of minIn and maxIn
    let minInFxP = DoubleAsFixedPoint(pIn, qIn, minIn);
    let maxInFxP = DoubleAsFixedPoint(pIn, qIn, maxIn);

    // Compute number of values in between minIn and maxIn
    let deltaIn = 1.0 / (2.0^IntAsDouble(qIn));
    let numValues = Truncate((maxInFxP - minInFxP) / deltaIn) + 1;

    // Go through each value, compute the number of integer bits required, and update pOut if it's bigger than
    // current pOut. We also store the output values since we will be using them when creating the output part of the
    // lookup table circuit
    mutable outValues = [0.0, size = numValues]; // List to store all the output values (initialized at all 0s)
    mutable inValueFxP = minInFxP; // Starting input value
    mutable pOut = 0; // Set initial pOut value which will be updated in loop below
    for i in 0..numValues-1 {
        // First a quick check to see that the enumaration is going correctly, i.e. that we are hitting all the values in order
        let inAddress = BoolArrayAsInt(FixedPointAsBoolArray(pIn, qIn, inValueFxP - minInFxP));
        Fact(inAddress == i, $"Unexpected address in enumeration");

        // Now we compute the output value, compute the number of integer bits it has and see if it is bigger than our current pOut
        let outValue = func(inValueFxP);
        set outValues w/= i <- outValue; // this is the syntax to say "outValues = outValues but with the ith index as outValue"
        set pOut = MaxI(pOut, BitSizeI(Ceiling(AbsD(outValue))) + 1); //the +1 is for the sign bit
        set inValueFxP += deltaIn;
    }

    //So we have now computed the number of integer bits for the output values. Now all that's left is to make the circuit!

    // We first create a list of FixedPoints with all the outValues
    let outValuesFxP = Mapped(DoubleAsFixedPoint(pOut, qOut, _), outValues);

    // Next we map outValuesFP to bitstrings
    let outBits = Mapped(FixedPointAsBoolArray(pOut, qOut, _), outValues);
    // Message($"{outBits}");

    // Now we use the fixed point approximation of the minimum value of the input
    // and the list of output bit values to make the operation lookupOperation: (FixedPoint, FixedPoint) => Unit
    // More comments on how that's done in within the function
    let lookupOperation = LookupOperationWrapper(minInFxP, outBits, _, _);


    return FunctionWithLookupTable(
        pIn,
        qIn,
        pOut,
        qOut,
        lookupOperation
    );
}

    /// # Summary
/// Creates a lookup table operation. This operation will require the minimum input value as a FixedPoint register,
/// the list of output values in bits,the FixedPoint register with the input value and the FixedPoint register that
/// will store the output value. Note that this imples that the bit size requirement of these registers are pre-computed
/// beforehand
///
/// # Input
/// ## minInFxp
/// The minimum possible value of the input to the lookup table
/// ## outBits
/// The list of output values in bits in order, where the first value is the output for the smallest input value and
/// the last value is the output for the largest input value
/// ## input
/// Qubit FixedPoint register containing input values
/// ## output
/// Qubit FixedPoint register containing where output values will be stored
internal operation LookupOperationWrapper(minInFxP : Double, outBits : Bool[][], input : FixedPoint, output : FixedPoint) : Unit is Adj {

    let integerBitsIn = input::IntegerBits;
    let registerIn = input::Register;
    let fractionalBitsIn = Length(registerIn) - integerBitsIn;

    // We are now creating the lookup table. If the smallest value (i.e. minInFxP) happens to be 0, then we can just use
    // the Select operation which implements the lookup table in ##. However, if the minimum value is not 0, then we want to first subtract
    // it, because the lookup table always assumes that the miminum value is 00...0 and the maximum value is 11...1 in incrementing order,
    // so we are re-defining the minimum number as represented by 00...0 and hence subracting the minimum from our value.
    // (We add the minimum back after making the lookup table)
    within {
        // Currently we always uncompute the lookup table garbage qubits, but we can think of making an option to remove the uncomputation (and keep the garbage qubits)
        if minInFxP != 0.0 {
            // Make a new fixed point register to store the minimum vlaue
            use minRegister = Qubit[Length(registerIn)];
            let minInReg = FixedPoint(integerBitsIn, minRegister); //
            within {
                PrepareFxP(minInFxP, minInReg); // Store minimum value in prepared register (automatically creates closest FxP approximation)
            } apply {
                SubtractFxP(input, minInReg); // SubtractFxP(a, b) : a <- a - b
            }
        }
    } apply {
        let n = Length(input::Register);
        let nRequired = Ceiling(Lg(IntAsDouble(Length(outBits))));
        Fact(nRequired <= n, "Too few address bits");
        let addressRegisterFitted = input::Register[...nRequired - 1];
        Select(outBits, input::Register[...nRequired - 1], output::Register);
    }
}

/// # Summary
/// Basically applies the ApplyPauliFromBitString function but with the extra check that the length of bits is the same
/// as the number of qubits (so nothing can be implicitly ignored)
internal operation WriteBits(bits : Bool[], qubitArray : Qubit[]) : Unit is Adj + Ctl {
    Fact(Length(bits) == Length(qubitArray), "Dimensions of bits and qubitArray should be the same");
    ApplyPauliFromBitString(PauliX, true, bits, qubitArray);
}

    /// # Summary
/// Helper function that creates an operator that takes in just 1 binary value input (i.e. a list
/// of booleans) and makes the circuit to apply paulis to create that binary value. We do this
/// so that we can use it as part of the Mapped function to be able to make a list of unitaries
/// given a list of binary numbers
internal function MakeWriteBitsUnitary(bits : Bool[]) : Qubit[] => Unit is Adj + Ctl {
    return WriteBits(bits, _);

}

    /// # Summary
/// This opration makes the lookup table by using the multiplex unitary operator - the operator that implements
/// different unitaries based on the value of the controlled bits. We just define each unitary as the set of
/// PauliX gates that will make the output qubit correspond to the data bits.
internal operation Select(data : Bool[][], addressRegister : Qubit[], outputRegister : Qubit[]) : Unit is Adj {

    let unitaries = Mapped(MakeWriteBitsUnitary, data);
    MultiplexOperations(unitaries, addressRegister, outputRegister);
}



// ----
import Signed.ApplyWithCA;

open Microsoft.Quantum.Arrays;
open Microsoft.Quantum.Convert;
open Microsoft.Quantum.Diagnostics;
open Microsoft.Quantum.Intrinsic;
open Microsoft.Quantum.Math;

/// # Summary
/// Applies a Pauli rotation conditioned on an array of qubits.
///
/// # Description
/// This applies a multiply controlled unitary operation that performs
/// rotations by angle $\theta_j$ about single-qubit Pauli operator $P$
/// when controlled by the $n$-qubit number state $\ket{j}$.
/// In particular, the action of this operation is represented by the
/// unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n - 1}_{j=0} \ket{j}\bra{j} \otimes e^{i P \theta_j}.
/// \end{align}
/// $$
///
/// # Input
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## pauli
/// Pauli operator $P$ that determines axis of rotation.
///
/// ## control
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Single qubit register that is rotated by $e^{i P \theta_j}$.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # See Also
/// - Microsoft.Quantum.Canon.ApproximatelyMultiplexPauli
operation MultiplexPauli(coefficients : Double[], pauli : Pauli, control : Qubit[], target : Qubit) : Unit is Adj + Ctl {
    ApproximatelyMultiplexPauli(0.0, coefficients, pauli, control, target);
}

    /// # Summary
/// Applies a Pauli rotation conditioned on an array of qubits, truncating
/// small rotation angles according to a given tolerance.
///
/// # Description
/// This applies a multiply controlled unitary operation that performs
/// rotations by angle $\theta_j$ about single-qubit Pauli operator $P$
/// when controlled by the $n$-qubit number state $\ket{j}$.
/// In particular, the action of this operation is represented by the
/// unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n - 1}_{j=0} \ket{j}\bra{j} \otimes e^{i P \theta_j}.
/// \end{align}
/// ##
///
/// # Input
/// ## tolerance
/// A tolerance below which small coefficients are truncated.
///
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## pauli
/// Pauli operator $P$ that determines axis of rotation.
///
/// ## control
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Single qubit register that is rotated by $e^{i P \theta_j}$.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # See Also
/// - Microsoft.Quantum.Canon.MultiplexPauli
operation ApproximatelyMultiplexPauli(tolerance : Double, coefficients : Double[], pauli : Pauli, control : Qubit[], target : Qubit) : Unit is Adj + Ctl {
    if pauli == PauliZ {
        let op = ApproximatelyMultiplexZ(tolerance, coefficients, control, _);
        op(target);
    } elif pauli == PauliX {
        let op = ApproximatelyMultiplexPauli(tolerance, coefficients, PauliZ, control, _);
        ApplyWithCA(H, op, target);
    } elif pauli == PauliY {
        let op = ApproximatelyMultiplexPauli(tolerance, coefficients, PauliX, control, _);
        ApplyWithCA(Adjoint S, op, target);
    } elif pauli == PauliI {
        ApproximatelyApplyDiagonalUnitary(tolerance, coefficients, control);
    } else {
        fail $"MultiplexPauli failed. Invalid pauli {pauli}.";
    }
}

    /// # Summary
/// Applies a Pauli Z rotation conditioned on an array of qubits.
///
/// # Description
/// This applies the multiply controlled unitary operation that performs
/// rotations by angle $\theta_j$ about single-qubit Pauli operator $Z$
/// when controlled by the $n$-qubit number state $\ket{j}$.
/// In particular, this operation can be represented by the unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n-1}_{j=0} \ket{j}\bra{j} \otimes e^{i Z \theta_j}.
/// \end{align}
/// $$
///
/// # Input
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## control
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Single qubit register that is rotated by $e^{i P \theta_j}$.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # References
/// - Synthesis of Quantum Logic Circuits
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
///   https://arxiv.org/abs/quant-ph/0406176
///
/// # See Also
/// - Microsoft.Quantum.Canon.ApproximatelyMultiplexZ
operation MultiplexZ(coefficients : Double[], control : Qubit[], target : Qubit) : Unit is Adj + Ctl {
    ApproximatelyMultiplexZ(0.0, coefficients, control, target);
}

internal function AnyOutsideToleranceD(tolerance : Double, coefficients : Double[]) : Bool {
    // NB: We don't currently use Any / Mapped for this, as we want to be
    //     able to short-circuit.
    for coefficient in coefficients {
        if AbsD(coefficient) >= tolerance {
            return true;
        }
    }
    return false;
}

internal function AnyOutsideToleranceCP(tolerance : Double, coefficients : ComplexPolar[]) : Bool {
    for coefficient in coefficients {
        if AbsComplexPolar(coefficient) > tolerance {
            return true;
        }
    }
    return false;
}

    /// # Summary
/// Applies a Pauli Z rotation conditioned on an array of qubits, truncating
/// small rotation angles according to a given tolerance.
///
/// # Description
/// This applies the multiply controlled unitary operation that performs
/// rotations by angle $\theta_j$ about single-qubit Pauli operator $Z$
/// when controlled by the $n$-qubit number state $\ket{j}$.
/// In particular, this operation can be represented by the unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n-1}_{j=0} \ket{j}\bra{j} \otimes e^{i Z \theta_j}.
/// \end{align}
/// $$
///
/// # Input
/// ## tolerance
/// A tolerance below which small coefficients are truncated.
///
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## control
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Single qubit register that is rotated by $e^{i P \theta_j}$.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # References
/// - Synthesis of Quantum Logic Circuits
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
///   https://arxiv.org/abs/quant-ph/0406176
///
/// # See Also
/// - Microsoft.Quantum.Canon.MultiplexZ
operation ApproximatelyMultiplexZ(tolerance : Double, coefficients : Double[], control : Qubit[], target : Qubit) : Unit is Adj + Ctl {
    body (...) {
        // pad coefficients length at tail to a power of 2.
        let coefficientsPadded = Padded(-2^Length(control), 0.0, coefficients);

        if Length(coefficientsPadded) == 1 {
            // Termination case
            if AbsD(coefficientsPadded[0]) > tolerance {
                Exp([PauliZ], coefficientsPadded[0], [target]);
            }
        } else {
            // Compute new coefficients.
            let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
            ApproximatelyMultiplexZ(tolerance, coefficients0, (Most(control)), target);
            if AnyOutsideToleranceD(tolerance, coefficients1) {
                within {
                    CNOT(Tail(control), target);
                } apply {
                    ApproximatelyMultiplexZ(tolerance, coefficients1, (Most(control)), target);
                }
            }
        }
    }

    controlled (controlRegister, ...) {
        // pad coefficients length to a power of 2.
        let coefficientsPadded = Padded(2^(Length(control) + 1), 0.0, Padded(-2^Length(control), 0.0, coefficients));
        let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
        ApproximatelyMultiplexZ(tolerance, coefficients0, control, target);
        if AnyOutsideToleranceD(tolerance, coefficients1) {
            within {
                Controlled X(controlRegister, target);
            } apply {
                ApproximatelyMultiplexZ(tolerance, coefficients1, control, target);
            }
        }
    }
}

    /// # Summary
/// Applies an array of complex phases to numeric basis states of a register
/// of qubits.
///
/// # Description
/// This operation implements a diagonal unitary that applies a complex phase
/// $e^{i \theta_j}$ on the $n$-qubit number state $\ket{j}$.
/// In particular, this operation can be represented by the unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n-1}_{j=0}e^{i\theta_j}\ket{j}\bra{j}.
/// \end{align}
/// $$
///
/// # Input
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
/// ## qubits
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # References
/// - Synthesis of Quantum Logic Circuits
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
///   https://arxiv.org/abs/quant-ph/0406176
///
/// # See Also
/// - Microsoft.Quantum.Canon.ApproximatelyApplyDiagonalUnitary
operation ApplyDiagonalUnitary(coefficients : Double[], qubits : Qubit[]) : Unit is Adj + Ctl {
    ApproximatelyApplyDiagonalUnitary(0.0, coefficients, qubits);
}

    /// # Summary
/// Applies an array of complex phases to numeric basis states of a register
/// of qubits, truncating small rotation angles according to a given
/// tolerance.
///
/// # Description
/// This operation implements a diagonal unitary that applies a complex phase
/// $e^{i \theta_j}$ on the $n$-qubit number state $\ket{j}$.
/// In particular, this operation can be represented by the unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n-1}_{j=0}e^{i\theta_j}\ket{j}\bra{j}.
/// \end{align}
/// $$
///
/// # Input
/// ## tolerance
/// A tolerance below which small coefficients are truncated.
///
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## qubits
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # References
/// - Synthesis of Quantum Logic Circuits
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
///   https://arxiv.org/abs/quant-ph/0406176
///
/// # See Also
/// - Microsoft.Quantum.Canon.ApplyDiagonalUnitary
operation ApproximatelyApplyDiagonalUnitary(tolerance : Double, coefficients : Double[], qubits : Qubit[]) : Unit is Adj + Ctl {
    if IsEmpty(qubits) {
        fail "operation ApplyDiagonalUnitary -- Number of qubits must be greater than 0.";
    }

    // pad coefficients length at tail to a power of 2.
    let coefficientsPadded = Padded(-2^Length(qubits), 0.0, coefficients);

    // Compute new coefficients.
    let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
    ApproximatelyMultiplexZ(tolerance, coefficients1, (Most(qubits)), Tail(qubits));

    if Length(coefficientsPadded) == 2 {
        // Termination case
        if AbsD(coefficients0[0]) > tolerance {
            Exp([PauliI], 1.0 * coefficients0[0], qubits);
        }
    } else {
        ApproximatelyApplyDiagonalUnitary(tolerance, coefficients0, (Most(qubits)));
    }
}

    /// # Summary
/// Implementation step of multiply-controlled Z rotations.
/// # See Also
/// - Microsoft.Quantum.Canon.MultiplexZ
internal function MultiplexZCoefficients(coefficients : Double[]) : (Double[], Double[]) {
    let newCoefficientsLength = Length(coefficients) / 2;
    mutable coefficients0 = [0.0, size = newCoefficientsLength];
    mutable coefficients1 = [0.0, size = newCoefficientsLength];

    for idxCoeff in 0..newCoefficientsLength - 1 {
        set coefficients0 w/= idxCoeff <- 0.5 * (coefficients[idxCoeff] + coefficients[idxCoeff + newCoefficientsLength]);
        set coefficients1 w/= idxCoeff <- 0.5 * (coefficients[idxCoeff] - coefficients[idxCoeff + newCoefficientsLength]);
    }

    return (coefficients0, coefficients1);
}

    /// # Summary
/// Applies an array of operations controlled by an array of number states.
///
/// That is, applies Multiply-controlled unitary operation $U$ that applies a
/// unitary $V_j$ when controlled by $n$-qubit number state $\ket{j}$.
///
/// $U = \sum^{2^n-1}_{j=0}\ket{j}\bra{j}\otimes V_j$.
///
/// # Input
/// ## unitaries
/// Array of up to $2^n$ unitary operations. The $j$th operation
/// is indexed by the number state $\ket{j}$ encoded in little-endian format.
///
/// ## index
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Generic qubit register that $V_j$ acts on.
///
/// # References
/// - Encoding Electronic Spectra in Quantum Circuits with Linear T Complexity
///   Ryan Babbush, Craig Gidney, Dominic W. Berry, Nathan Wiebe, Jarrod McClean, Alexandru Paler, Austin Fowler, Hartmut Neven
///   https://arxiv.org/abs/1805.03662
operation MultiplexOperations<'T>(unitaries : ('T => Unit is Adj + Ctl)[], index : Qubit[], target : 'T) : Unit is Adj + Ctl {
    body (...) {
        let (N, n) = DimensionsForMultiplexer(Length(unitaries), index);

        if N == 1 {
            // base case
            Head(unitaries)(target);
        } else {
            let (most, tail) = MostAndTail(index[...n - 1]);
            let parts = Partitioned([2^(n - 1)], unitaries);

            within {
                X(tail);
            } apply {
                Controlled MultiplexOperations([tail], (parts[0], (most), target));
            }

            Controlled MultiplexOperations([tail], (parts[1], (most), target));
        }
    }

    controlled (ctls, ...) {
        let nCtls = Length(ctls);

        if nCtls == 0 {
            MultiplexOperations(unitaries, index, target);
        } elif nCtls == 1 {
            let (N, n) = DimensionsForMultiplexer(Length(unitaries), index);

            let ctl = Head(ctls);

            if N == 1 {
                // base case
                Controlled (Head(unitaries))(ctls, target);
            } else {
                use helper = Qubit();

                let (most, tail) = MostAndTail(index[...n - 1]);
                let parts = Partitioned([2^(n - 1)], unitaries);

                within {
                    X(tail);
                } apply {
                    ApplyAnd(ctl, tail, helper);
                }

                Controlled MultiplexOperations([helper], (parts[0], (most), target));

                CNOT(ctl, helper);

                Controlled MultiplexOperations([helper], (parts[1], (most), target));

                Adjoint ApplyAnd(ctl, tail, helper);
            }
        } else {
            use helper = Qubit();
            within {
                Controlled X(ctls, helper);
            } apply {
                Controlled MultiplexOperations([helper], (unitaries, index, target));
            }
        }
    }
}

    /// # Summary
/// Validates and adjusts dimensions for address register
///
/// # Description
/// Given $N$ unitaries in `numUnitaries` and an address register of length $n'$,
/// this function first checks whether $N \neq 0$ and $\lceil\log_2 N\rceil = n \le n'$,
/// and then returns the tuple $(N, n)$.
///
/// # Input
/// ## numUnitaries
/// The number of unitaries to multiplex.
/// ## address
/// The address register.
internal function DimensionsForMultiplexer(numUnitaries : Int, address : Qubit[]) : (Int, Int) {
    let N = numUnitaries;
    Fact(N > 0, "data cannot be empty");

    let n = Ceiling(Lg(IntAsDouble(N)));
    Fact(Length(address) >= n, $"address register is too small, requires at least {n} qubits");

    return (N, n);
}



    /// # Summary
    /// Inverts a given target qubit if and only if both control qubits are in the 1 state,
    /// using measurement to perform the adjoint operation.
    ///
    /// # Description
    /// Inverts `target` if and only if both controls are 1, but assumes that
    /// `target` is in state 0.  The operation has T-count 4, T-depth 2 and
    /// requires no helper qubit, and may therefore be preferable to a CCNOT
    /// operation, if `target` is known to be 0.  The adjoint of this operation
    /// is measurement based and requires no T gates.
    ///
    /// The controlled application of this operation requires no helper qubit,
    /// `2^c` `Rz` gates and is not optimized for depth, where `c` is the number
    /// of overall control qubits including the two controls from the `ApplyAnd`
    /// operation.  The adjoint controlled application requires `2^c - 1` `Rz`
    /// gates (with an angle twice the size of the non-adjoint operation), no
    /// helper qubit and is not optimized for depth.
    ///
    /// # Input
    /// ## control1
    /// First control qubit
    /// ## control2
    /// Second control qubit
    /// ## target
    /// Target auxiliary qubit; must be in state 0
    ///
    /// # References
    /// - Cody Jones: "Novel constructions for the fault-tolerant Toffoli gate",
    ///   Phys. Rev. A 87, 022328, 2013
    ///   [arXiv:1212.5069](https://arxiv.org/abs/1212.5069)
    ///   doi:10.1103/PhysRevA.87.022328
    /// - Craig Gidney: "Halving the cost of quantum addition", Quantum 2, page
    ///   74, 2018
    ///   [arXiv:1709.06648](https://arxiv.org/abs/1709.06648)
    ///   doi:10.1103/PhysRevA.85.044302
    /// - Mathias Soeken: "Quantum Oracle Circuits and the Christmas Tree Pattern",
    ///   [Blog article from December 19, 2019](https://msoeken.github.io/blog_qac.html)
    ///   (note: explains the multiple controlled construction)
    operation ApplyAnd(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit {
        body (...) {
            AssertAllZero([target]);
            H(target);
            T(target);
            CNOT(control1, target);
            CNOT(control2, target);
            within {
                CNOT(target, control1);
                CNOT(target, control2);
            }
            apply {
                Adjoint T(control1);
                Adjoint T(control2);
                T(target);
            }
            HY(target);
        }
        adjoint (...) {
            H(target);
            AssertMeasurementProbability([PauliZ], [target], One, 0.5, "Probability of the measurement must be 0.5", 1e-10);
            if (IsResultOne(MResetZ(target))) {
                CZ(control1, control2);
            }
        }
        controlled (controls, ...) {
            ApplyMultiplyControlledAnd(controls + [control1, control2], target);
        }
        adjoint controlled (controls, ...) {
            Adjoint ApplyMultiplyControlledAnd(controls + [control1, control2], target);
        }
    }


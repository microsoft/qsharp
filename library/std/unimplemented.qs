// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Math {
    /// ModPowL is not supported. Use ExpModL instead.
    @Unimplemented()
    function ModPowL (value : BigInt, exponent : BigInt, modulus : BigInt) : BigInt {
        fail "";
    }

    /// ExpD(a) is not supported. Use E()^a instead.
    @Unimplemented()
    function ExpD (a : Double) : Double {
        fail "";
    }

    /// HalfIntegerBinom has been removed. Provide your own implementation if needed.
    @Unimplemented()
    function HalfIntegerBinom (k : Int) : Double {
        fail "";
    }

    /// IEEERemainder has been removed. Provide your own implementation if needed.
    @Unimplemented()
    function IEEERemainder (x : Double, y : Double) : Double {
        fail "";
    }

    /// Microsoft.Quantum.Math.ComplexPolarAsComplex has been moved. Use Microsoft.Quantum.Convert.ComplexPolarAsComplex instead.
    @Unimplemented()
    function ComplexPolarAsComplex (input : ComplexPolar) : Complex {
        fail "";
    }

    /// Microsoft.Quantum.Math.ComplexAsComplexPolar has been moved. Use Microsoft.Quantum.Convert.ComplexAsComplexPolar instead.
    @Unimplemented()
    function ComplexAsComplexPolar (input : Complex) : ComplexPolar {
        fail "";
    }
}

namespace Microsoft.Quantum.Arithmetic {

    /// Microsoft.Quantum.Arithmetic.ApplyXorInPlace has been moved. Use Microsoft.Quantum.Canon.ApplyXorInPlace instead.
    @Unimplemented()
    operation ApplyXorInPlace(value : Int, target : Qubit[]) : Unit is Adj + Ctl {
        fail "";
    }

    /// Microsoft.Quantum.Arithmetic.MeasureInteger has been moved. Use Microsoft.Quantum.Measurement.MeasureInteger instead.
    @Unimplemented()
    operation MeasureInteger(target : Qubit[]) : Int {
        fail "";
    }

    /// Explicit LittleEndian type is not supported. Use Qubit[] directly since all library functions use little-endian format for qubit registers.
    @Unimplemented()
    newtype LittleEndian = Qubit[];

}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Math {
    /// ModPowL is not supported. Use ExpModL instead.
    @Unimplemented()
    function ModPowL (value : BigInt, exponent : BigInt, modulus : BigInt) : BigInt {
        fail "";
    }

    /// ExpD is not supported. Use E()^a instead.
    @Unimplemented()
    function ExpD (a : Double) : Double {
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

    /// LittleEndian type is not supported. All standart library functions use little-endian format for qubit registers `Qubit[]`.
    @Unimplemented()
    newtype LittleEndian = Qubit[];

}

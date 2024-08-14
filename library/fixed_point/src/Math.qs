// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


open Microsoft.Quantum.Convert;

/// # Summary
/// Returns the smallest representable number for specific fixed point dimensions.
///
/// # Input
/// ## integerBits
/// Number of integer bits (including the sign bit).
/// ## fractionalBits
/// Number of fractional bits.
///
/// # Remark
/// The value can be computed as $-2^{p-1}$, where $p$ is the number of integer bits.
function SmallestFixedPoint(integerBits : Int, fractionalBits : Int) : Double {
    return -(2.0 ^ IntAsDouble(integerBits - 1));
}

/// # Summary
/// Returns the largest representable number for specific fixed point dimensions.
///
/// # Input
/// ## integerBits
/// Number of integer bits (including the sign bit).
/// ## fractionalBits
/// Number of fractional bits.
///
/// # Remark
/// The value can be computed as $2^{p-1} - 2^{-q}$, where $p$
/// is the number of integer bits and $q$ is the number of fractional bits.
function LargestFixedPoint(integerBits : Int, fractionalBits : Int) : Double {
    return (2.0 ^ IntAsDouble(integerBits - 1)) - (2.0 ^ (-IntAsDouble(fractionalBits)));
}


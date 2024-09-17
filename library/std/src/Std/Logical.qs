// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


/// # Summary
/// Returns the boolean exclusive disjunction (eXclusive OR, XOR)
/// of two input boolean values.
///
/// # Input
/// ## first
/// The first boolean value to be considered.
///
/// ## second
/// The second boolean value to be considered.
///
/// # Output
/// A `Bool` which is `true` if and only if exactly one of `first` and `second` is `true`.
///
/// # Remarks
/// In Q#, `Xor(a, b)` is equivalent to `a != b`.
///
/// # Example
/// ```qsharp
/// let result = Xor(true, false);
/// // result is true
/// ```
function Xor(first : Bool, second : Bool) : Bool {
    first != second
}
export Xor;


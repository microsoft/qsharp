// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arrays {

    /// Given an array, returns a range over the indices of that array, suitable
    /// for use in a for loop.
    function IndexRange<'TElement>(array : 'TElement[]) : Range {
       return 0 .. array::Length - 1;
    }

}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


// This file re-exports the standard library under the name `Std`, which will be the preferred standard library API going forward.

namespace Microsoft.Quantum {
    export Std.Arrays;
}

namespace Std {
    export
        Microsoft.Quantum.Canon,
        Microsoft.Quantum.Convert,
        Microsoft.Quantum.Core,
        Microsoft.Quantum.Diagnostics,
        Microsoft.Quantum.Logical,
        Microsoft.Quantum.Intrinsic,
        Microsoft.Quantum.Math,
        Microsoft.Quantum.Measurement,
        Microsoft.Quantum.Random,
        Microsoft.Quantum.ResourceEstimation;
}

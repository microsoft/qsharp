// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


// This file re-exports the standard library under the name `Std`, which will be the preferred standard library API going forward.

namespace Microsoft.Quantum {
    export Std.Arrays, Std.Convert, Std.Diagnostics, Std.Logical, Std.Math, Std.Measurement, Std.Intrinsic, Std.Random, Std.ResourceEstimation, Std.Canon, Std.Core;
}

namespace Microsoft.Quantum.Unstable {
    export Std.Unstable.Arithmetic, Std.Unstable.TableLookup, Std.Unstable.StatePreparation;
}


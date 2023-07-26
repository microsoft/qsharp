// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Random {

    /// # Summary
    /// Draws a random integer in a given inclusive range.
    ///
    /// # Input
    /// ## min
    /// The smallest integer to be drawn.
    /// ## max
    /// The largest integer to be drawn.
    ///
    /// # Output
    /// An integer in the inclusive range from `min` to `max` with uniform
    /// probability.
    ///
    /// # Remarks
    /// Fails if `max < min`.
    ///
    /// # Example
    /// The following Q# snippet randomly rolls a six-sided die:
    /// ```qsharp
    /// let roll = DrawRandomInt(1, 6);
    /// ```
    operation DrawRandomInt(min : Int, max : Int) : Int {
        body intrinsic;
    }

    /// # Summary
    /// Draws a random real number in a given inclusive interval.
    ///
    /// # Input
    /// ## min
    /// The smallest real number to be drawn.
    /// ## max
    /// The largest real number to be drawn.
    ///
    /// # Output
    /// A random real number in the inclusive interval from `min` to `max` with
    /// uniform probability.
    ///
    /// # Remarks
    /// Fails if `max < min`.
    ///
    /// # Example
    /// The following Q# snippet randomly draws an angle between $0$ and $2 \pi$:
    /// ```qsharp
    /// let angle = DrawRandomDouble(0.0, 2.0 * PI());
    /// ```
    operation DrawRandomDouble(min : Double, max : Double) : Double {
        body intrinsic;
    }

}

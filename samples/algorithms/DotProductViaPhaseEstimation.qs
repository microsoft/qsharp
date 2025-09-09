// MIT License

// Copyright (c) 2023 KPMG Australia

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
import Std.Math.*;
import Std.Convert.*;

@EntryPoint(Adaptive_RI)
operation Main() : (Int, Int) {
    // The angles for inner product. Inner product is meeasured for vectors
    // (cos(Θ₁/2), sin(Θ₁/2)) and (cos(Θ₂/2), sin(Θ₂/2)).
    // Number of iterations
    // Perform measurements
    Message("Computing inner product of vectors (cos(Θ₁/2), sin(Θ₁/2))⋅(cos(Θ₂/2), sin(Θ₂/2)) ≈ -cos(x𝝅/2ⁿ)");
    let result = PerformMeasurements();
    // Return result
    return (result, 4);
}

@Config(Adaptive)
@Config(not HigherLevelConstructs)
@Config(not FloatingPointComputations)
operation PerformMeasurements() : Int {
    // n = 4, so measurementCount = n + 1 = 5
    return QuantumInnerProduct();
}

@Config(HigherLevelConstructs)
@Config(FloatingPointComputations)
operation PerformMeasurements() : Int {
    let theta1 = PI() / 7.0;
    let theta2 = PI() / 5.0;
    Message($"Θ₁={theta1}, Θ₂={theta2}.");

    // First compute quantum approximation
    // n = 4 so measurementCount = 5 and 2^n = 16
    let x = QuantumInnerProduct();
    let angle = PI() * IntAsDouble(x) / 16.0;
    let computedInnerProduct = -Cos(angle);
    Message($"x = {x}, n = 4.");

    // Now compute true inner product
    let trueInnterProduct = ClassicalInnerProduct();

    Message($"Computed value = {computedInnerProduct}, true value = {trueInnterProduct}");

    return x;
}

function ClassicalInnerProduct() : Double {
    let theta1 = PI() / 7.0;
    let theta2 = PI() / 5.0;
    return Cos(theta1 / 2.0) * Cos(theta2 / 2.0) + Sin(theta1 / 2.0) * Sin(theta2 / 2.0);
}

operation QuantumInnerProduct() : Int {
    //Create target register
    use TargetReg = Qubit();
    //Create ancilla register
    use AncilReg = Qubit();
    //Run iterative phase estimation
    let Results = IterativePhaseEstimation(TargetReg, AncilReg);
    Reset(TargetReg);
    Reset(AncilReg);
    return Results;
}

operation IterativePhaseEstimation(
    TargetReg : Qubit,
    AncilReg : Qubit
) : Int {

    let Measurements = 5; // previously iterationCount (n + 1) with n = 4

    let theta1 = PI() / 7.0;
    let theta2 = PI() / 5.0;

    use ControlReg = Qubit();
    mutable MeasureControlReg = [Zero, size = Measurements];
    mutable bitValue = 0;
    //Apply to initialise state, this is defined by the angles theta1 and theta2
    StateInitialisation(TargetReg, AncilReg);
    for index in 0..Measurements - 1 {
        H(ControlReg);
        //Don't apply rotation on first set of oracles
        if index > 0 {
            //Loop through previous results
            for index2 in 0..index - 1 {
                if MeasureControlReg[Measurements - 1 - index2] == One {
                    //Rotate control qubit dependent on previous measurements and number of measurements
                    let angle = -IntAsDouble(2^(index2)) * PI() / (2.0^IntAsDouble(index));
                    R(PauliZ, angle, ControlReg);
                }
            }

        }
        let powerIndex = (1 <<< (Measurements - 1 - index));
        //Apply a number of oracles equal to 2^index, where index is the number or measurements left
        for _ in 1..powerIndex {
            Controlled GOracle([ControlReg], (TargetReg, AncilReg));
        }
        H(ControlReg);
        //Make a measurement mid circuit
        set MeasureControlReg w/= (Measurements - 1 - index) <- MResetZ(ControlReg);
        if MeasureControlReg[Measurements - 1 - index] == One {
            //Assign bitValue based on previous measurement
            bitValue += 2^(index);
        }
    }
    return bitValue;
}

/// # Summary
/// This is state preperation operator A for encoding the 2D vector (page 7)
operation StateInitialisation(
    TargetReg : Qubit,
    AncilReg : Qubit
) : Unit is Adj + Ctl {

    let theta1 = PI() / 7.0;
    let theta2 = PI() / 5.0;

    H(AncilReg);
    // Arbitrary controlled rotation based on theta. This is vector v.
    Controlled R([AncilReg], (PauliY, theta1, TargetReg));
    // X gate on ancilla to change from |+〉 to |-〉.
    X(AncilReg);
    // Arbitrary controlled rotation based on theta. This is vector c.
    Controlled R([AncilReg], (PauliY, theta2, TargetReg));
    X(AncilReg);
    H(AncilReg);
}

operation GOracle(
    TargetReg : Qubit,
    AncilReg : Qubit
) : Unit is Adj + Ctl {

    // Angles inlined

    Z(AncilReg);
    within {
        Adjoint StateInitialisation(TargetReg, AncilReg);
        X(AncilReg);
        X(TargetReg);
    } apply {
        Controlled Z([AncilReg], TargetReg);
    }
}

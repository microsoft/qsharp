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

namespace IterativePhaseEstimation {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    @EntryPoint()
    operation Main() : (Int, Int) {
        // The angles for inner product
        let theta1 = PI()/7.0;
        let theta2 = PI()/5.0;
        // Number of iterations
        let n = 4;
        // Perform measurements
        let result = PerformMeasurements(theta1, theta2, n);
        // Return result
        return (result, n);
    }

    @Config(Adaptive)
    operation PerformMeasurements(theta1: Double, theta2: Double, n: Int) : Int {
        Message("Inner product is -cos(x𝝅/2ⁿ).");
        let measurementCount = n + 1;
        return QuantumInnerProduct(theta1, theta2, measurementCount);
    }

    @Config(Unrestricted)
    operation PerformMeasurements(theta1: Double, theta2: Double, n: Int) : Int {
        Message($"Computing inner product for unit vectors with theta_1={theta1}, theta_2={theta2}.");

        // First compute quantum approximation
        let measurementCount = n + 1;
        let x = QuantumInnerProduct(theta1, theta2, measurementCount);
        let angle = PI() * IntAsDouble(x) / IntAsDouble(2 ^ n);
        let computedInnerProduct = -Cos(angle);
        Message($"Measured value x = {x}, n = {n}. Inner product is -cos(x𝝅/2ⁿ).");

        // Now compute true inner product
        let trueInnterProduct = ClassicalInnerProduct(theta1, theta2);

        Message($"Computed value = {computedInnerProduct}, true value = {trueInnterProduct}");

        return x;
    }

    operation ClassicalInnerProduct(theta1: Double, theta2: Double) : Double {
        return Cos(theta1/2.0)*Cos(theta2/2.0)+Sin(theta1/2.0)*Sin(theta2/2.0);
    }

    operation QuantumInnerProduct(theta1: Double, theta2: Double, iterationCount: Int) : Int {
        //Create target register
        use TargetReg = Qubit();
        //Create ancilla register
        use AncilReg = Qubit();
        //Run iterative phase estimation
        let Results = IterativePhaseEstimation(TargetReg, AncilReg, theta1, theta2, iterationCount);
        Reset(TargetReg);
        Reset(AncilReg);
        return Results;
    }

    operation IterativePhaseEstimation(
        TargetReg : Qubit,
        AncilReg : Qubit,
        theta1 : Double,
        theta2 : Double,
        Measurements : Int) : Int {

        use ControlReg = Qubit();
        mutable MeasureControlReg = [Zero, size = Measurements];
        mutable bitValue = 0;
        //Apply to initialise state, this is defined by the angles theta_1 and theta_2
        StateInitialisation(TargetReg, AncilReg, theta1, theta2);                                     
        for index in 0 .. Measurements - 1 {                                                             
            H(ControlReg);
            //Don't apply rotation on first set of oracles                                                                              
            if index > 0 {                      
                //Loop through previous results                                                        
                for index2 in 0 .. index - 1 {                                                           
                    if MeasureControlReg[Measurements - 1 - index2] == One {
                        //Rotate control qubit dependent on previous measurements and number of measurements 
                        let angle = -IntAsDouble(2^(index2))*PI()/(2.0^IntAsDouble(index));           
                        R(PauliZ, angle, ControlReg);                                                  
                    }
                }
                
            }
            let powerIndex = (1 <<< (Measurements - 1 - index));
            //Apply a number of oracles equal to 2^index, where index is the number or measurements left
            for _ in 1 .. powerIndex{
                    Controlled GOracle([ControlReg],(TargetReg, AncilReg, theta1, theta2));
                }
            H(ControlReg);
            //Make a measurement mid circuit
            set MeasureControlReg w/= (Measurements - 1 - index) <- MResetZ(ControlReg);
            if MeasureControlReg[Measurements - 1 - index] == One{
                //Assign bitValue based on previous measurement
                set bitValue += 2^(index);
            }
        }
        return bitValue;
    }

    /// # Summary
    /// This is state preperation operator A for encoding the 2D vector (page 7)
    operation StateInitialisation(
        TargetReg : Qubit,
        AncilReg : Qubit,
        theta1 : Double,
        theta2 : Double) : Unit is Adj + Ctl {

        H(AncilReg);
        // Arbitrary controlled rotation based on theta. This is vector v.
        Controlled R([AncilReg], (PauliY, theta1, TargetReg));        
        // X gate on ancilla to change from |+> to |->.                                    
        X(AncilReg);
        // Arbitrary controlled rotation based on theta. This is vector c.                                                   
        Controlled R([AncilReg], (PauliY, theta2, TargetReg));        
        X(AncilReg);                                                  
        H(AncilReg);                                                  
    }

    operation GOracle(
        TargetReg : Qubit,
        AncilReg : Qubit,
        theta1 : Double,
        theta2 : Double) : Unit is Adj + Ctl {

        Z(AncilReg);
        within {
            Adjoint StateInitialisation(TargetReg, AncilReg, theta1, theta2);
            X(AncilReg);                                                        
            X(TargetReg);
        }   
        apply {
            Controlled Z([AncilReg],TargetReg);
        }
    }
}

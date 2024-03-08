namespace Kata { 
    open Microsoft.Quantum.Math;

    operation ComplexModulus(x : Complex) : Double {
        
        let (a,b) = x!;
        let m = (a^2.0 + b^2.0)^0.5;

        return (m);
    }
}

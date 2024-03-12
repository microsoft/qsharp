namespace Kata {    
    open Microsoft.Quantum.Math;
    
    function ComplexPolarMult(x : ComplexPolar, y: ComplexPolar) : ComplexPolar {
        let (r1, theta1) = x!;
        let (r2, theta2) = y!;
        mutable theta3 = theta1 + theta2;
        if theta3 > PI() {
            set theta3 -= 2.0 * PI();
        }
        if theta3 < -PI() {
            set theta3 += 2.0 * PI();
        }
        return ComplexPolar(r1 * r2, theta3);
    }
}

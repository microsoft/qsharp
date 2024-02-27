namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Math;

    operation ComplexAddExp(x : Complex, y : Complex) : (Complex)  {
        
        return (x::Real + y::Real, x::Imag + y::Imag);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        mutable i = 1;
        mutable n=0;
        mutable success = true;
        repeat {   
            set n = 2*i;
            let expected = PowersOfIExp(n);
            let actual  = Kata.PowersOfI(n);
            if expected != actual {set success = false;}
            set i += 1;   
        }
        until (i > 25) or (success == false);   
        if success == true {Message("Success!");}
        else{Message("Solution failed.");}
        return success;
    }
}

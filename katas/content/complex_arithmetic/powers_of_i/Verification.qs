namespace Kata.Verification {
   
   operation PowersOfI_Reference(n : Int) : Int{
    // If n is divisible by 4
       if n % 4 == 0 { return 1;
       }
       else { return -1;
       }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        
        for count in 0 .. 24 {   
            let  n = 2*count;
            let expected = PowersOfI_Reference(n);
            let actual  = Kata.PowersOfI(n);
            if expected != actual{
              Message($"Incorrect solution. When n = {n} the result of exponentiation doesn't match expected value.");
              Message($"Actual value:{actual}. Expected value:{expected}");
              return false; 
            }
        }
        Message("Correct!");
        return true; 
    }
}

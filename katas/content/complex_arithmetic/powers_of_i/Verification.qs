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
        mutable i = 1;
        mutable n=0;
        mutable success = true;
        mutable actual = 0;
        mutable expected = 0;

        repeat {   
            set n = 2*i;
            set expected = PowersOfI_Reference(n);
            set actual  = Kata.PowersOfI(n);
            if expected != actual {set success = false;}
            set i += 1;   
        }
        until (i > 25) or (success == false);   
        if success == true { Message("Correct!");
        }
        else { Message("Incorrect solution. Result of exponentiation doesn't match expected value");
               Message($"Actual value: {actual}. Expected value: {expected}");
        }
        return success;
    }
}

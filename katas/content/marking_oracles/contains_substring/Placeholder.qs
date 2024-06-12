namespace Kata {
    operation Oracle_ContainsSubstring (x : Qubit[], y : Qubit, r : Bool[]) : Unit is Adj + Ctl {
        // Implement your solution here...

    }  

    // You might find this helper operation from an earlier task useful.  
    operation Oracle_ContainsSubstringAtPosition (x : Qubit[], y : Qubit, r : Bool[], p : Int) : Unit is Adj + Ctl {
        ApplyControlledOnBitString(r, X, x[p .. p + Length(r) - 1], y);
    }     
}

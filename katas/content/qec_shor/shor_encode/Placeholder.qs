namespace Kata {
    operation ShorEncode (qs : Qubit[]) : Unit is Adj + Ctl {
        // Implement your solution here...
        
    }

    // You might find this helper operation from an earlier task useful.
    operation BitflipEncode (qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
    }
}
namespace Kata {
    operation EvenOddNumbersSuperposition(qs : Qubit[], isEven : Bool) : Unit is Adj + Ctl {
        let N = Length(qs);

        for i in 0 .. N-2 {
            H(qs[i]);
        }

        // for odd numbers, flip the last bit to 1
        if not isEven {
            X(qs[N-1]);
        }
    }
}

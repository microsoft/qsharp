namespace Kata {
    operation ControlledRotation (qs : Qubit[], theta : Double) : Unit is Adj + Ctl {
        let control = qs[0];
        let target = qs[1];
        Controlled Rx([control], (theta, target));
    }
}
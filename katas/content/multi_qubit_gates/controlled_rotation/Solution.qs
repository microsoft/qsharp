namespace Kata {
    operation ControlledRotation (qs : Qubit[], theta : Double) : Unit is Adj + Ctl {
        let controll = qs[0];
        let target = qs[1];
        Controlled Rx([controll], (theta, target));
    }
}
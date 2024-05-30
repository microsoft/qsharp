namespace Kata {
    operation DistinguishYfromXZWithPhases (unitary : (Qubit => Unit is Adj+Ctl)) : Int {
        use (control, target) = (Qubit(), Qubit());
        // Distinguish Y from iY
        within {
            H(control);
        } apply {
            Controlled unitary([control], target);
            Controlled unitary([control], target);
        }
        Reset(target);
        let isY = MResetZ(control) == Zero;

        // Distinguish Y from -Y and iY from -iY
        within {
            H(control);
        } apply {
            Controlled unitary([control], target);
            // apply controlled variant of the gate we're expecting to compensate effect on target qubit
            if isY {
                CY(control, target);
            } else {
                CZ(control, target);
                CX(control, target);
            }
        }
        let result = isY ? M(control) == Zero ? 0 | 2 
                         | M(control) == Zero ? 1 | 3;
        Reset(control);
        return result;
        
    }
}

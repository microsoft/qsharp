import SubFolder.Circuit2.Circuit2;
import Circuit1.Circuit1;
operation Main() : Unit {
    use qs = Qubit[2];
    Circuit1(qs);
    Circuit2(qs);
}

export
    Circuit1 as Circuit1,
    Circuit2 as Circuit2;

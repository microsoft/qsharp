// OpenQASM simple quantum teleportation sample
//
// This OpenQASM program demonstrates how to teleport quantum state
// by communicating two classical bits and using previously entangled qubits.
// This code teleports one specific state, but any state can be teleported.

OPENQASM 3.0;
include "stdgates.inc";

// Allocate `qAlice`, `qBob` qubits
qubit qAlice;
qubit qBob;

// Reset and entangle `qAlice`, `qBob` qubits
reset qAlice;
reset qBob;
h qAlice;
cx qAlice, qBob;

// From now on qubits `qAlice` and `qBob` will not interact directly.

// Allocate `qToTeleport` qubit and prepare it to be |𝜓⟩≈0.9394|0⟩−0.3429𝑖|1⟩
qubit qToTeleport;
reset qToTeleport;
rx(0.7) qToTeleport;

// Prepare the message by entangling `qToTeleport` and `qAlice` qubits
cx qToTeleport, qAlice;
h qToTeleport;

// Obtain classical measurement results b1 and b2 at Alice's site.
bit b1 = measure qToTeleport;
bit b2 = measure qAlice;

// At this point classical bits b1 and b2 are "sent" to the Bob's site.

// Decode the message by applying adjustments based on classical data b1 and b2.
if (b1) {
    z qBob;
}
if (b2) {
    x qBob;
}

// Obtained messages should be |𝜓⟩≈0.9394|0⟩−0.3429𝑖|1⟩
// Rotate back to |0⟩ state and measure
rx(-0.7) qBob;
output bit shouldBeZero;
shouldBeZero = measure qBob;

// Each execution of this program should result in Zero.

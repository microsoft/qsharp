// Simulation of a simple Ising model evolution
// on a 1D grid with first-order Trotterization.
//
// This OpenQASM sample demonstrates simulation of an Ising model Hamiltonian
// on 1D grid of size N using a first-order Trotter-Suzuki approximation.
// This sample can be easily simulated classically with the grid of size 9
// and 1000 shots. This sample is suitable for Base Profile.
// For the purpose of simplicity this sample intentionally doesn't
// post-process results or perform eigenvalue estimation.

OPENQASM 3;
include "stdgates.inc";

// The size of a 1D grid is N
const int N = 9;

/// Simulate simple Ising model evolution
//
// # Description
// Simulates state |𝜓⟩ evolution to find |𝜓(t)⟩=U(t)|𝜓(0)⟩.
// |𝜓(0)⟩ is taken to be |0...0⟩.
// U(t)=e⁻ⁱᴴᵗ, where H is an Ising model Hamiltonian H = -J·Σ'ᵢⱼZᵢZⱼ + g·ΣᵢXᵢ
// Here Σ' is taken over all pairs of neighboring qubits <i,j>.
// Simulation is done by performing K steps assuming U(t)≈(U(t/K))ᴷ.
def IsingModel1DEvolution(
    float J,
    float g,
    float evolutionTime,
    int numberOfSteps,
    qubit[N] qs
) -> bit[N] {

    float dt = evolutionTime / numberOfSteps;
    
    angle theta_x = -g * dt;
    angle theta_zz = J * dt;

    reset qs;

    // Perform K steps
    for int i in [1:numberOfSteps] {

        // Single-qubit interaction with external field
        for int j in [0:N-1] {
            rx(theta_x * 2) qs[j];
        }

        // All of the following Rzz gates commute. So we apply them between "even"
        // pairs first and then between "odd" pairs to reduce the algorithm depth.

        // Interactions between "even" pairs
        for int j in [0:2:N-2] {
            rzz(theta_zz * 2) qs[j], qs[j + 1];
        }

        // Interactions between "odd" pairs
        for int j in [1:2:N-2] {
            rzz(theta_zz * 2) qs[j], qs[j + 1];
        }
    }

    bit[N] result = measure qs;
    return result;
}

// Main program

// Allocate qubit grid
qubit[N] qubits;

// Total evolution time
float evolutionTime = 4;
// Number of steps
int numberOfSteps = 7;

// Coefficient for 2-qubit interactions between neighboring qubits
float J = 1.0;
// Coefficient for external field interaction for individual qubits
float g = 0.7;

output bit[N] result;
result = IsingModel1DEvolution(J, g, evolutionTime, numberOfSteps, qubits);

## Relative and Global Phase

You may recall that a complex number has a parameter called its phase. If a complex number $x$ is written in polar form $x = re^{i\theta}$, its phase is $\theta$.

The phase of a basis state is the complex phase of the amplitude of that state. For example, a system in state $\frac{1 + i}{2}|0\rangle + \frac{1 - i}{2}|1\rangle$, the phase of $|0\rangle$ is $\frac{\pi}{4}$, and the phase of $|1\rangle$ is $-\frac{\pi}{4}$. The difference between these two phases is known as **relative phase**.

Multiplying the state of the entire system by $e^{i\theta}$ doesn't affect the relative phase: $\alpha|0\rangle + \beta|1\rangle$ has the same relative phase as $e^{i\theta}\big(\alpha|0\rangle + \beta|1\rangle\big)$. In the second expression, $\theta$ is known as the system's **global phase**.

The state of a qubit (or, more generally, the state of a quantum system) is defined by its relative phase - global phase arises as a consequence of using linear algebra to represent qubits, and has no physical meaning. That is, applying a phase to the entire state of a system (multiplying the entire vector by $e^{i\theta}$ for any real $\theta$) doesn't actually affect the state of the system. Because of this, global phase is sometimes known as **unobservable phase** or **hidden phase**.

# Phase Estimation

@[section]({
    "id": "phase_estimation__overview",
    "title": "Overview"
})

This kata introduces you to the phase estimation algorithm - an important building block in more advanced quantum algorithms such as integer factoring.

**This kata covers the following topics:**

- The definition of eigenvalues and eigenvectors
- The phase estimation problem
- The quantum phase estimation algorithm based on quantum Fourier transform

**What you should know to start working on this kata:**

- Basic quantum gates and measurements.
- Quantum Fourier transform.

@[section]({
    "id": "phase_estimation__eigen",
    "title": "Eigenvectors, Eigenvalues, and Eigenphases"
})

An *eigenvector* of a matrix $A$ is a non-zero vector that, when multiplied by that matrix, changes by a scalar factor:

$$A \ket{v} = \lambda \ket{v}$$

The number $\lambda$ is called an *eigenvalue* that corresponds to this eigenvector. In general, eigenvalues of matrices can be complex numbers.

Recall that all quantum gates are unitary matrices, for which their inverse equals their adjoint ($U^{-1} = U^\dagger$). This means that the eigenvalues of their eigenvectors have the property that their modulus equals $1$: 

$$|\lambda| = 1$$

Thus, they can be written in the following form:
$$\lambda = e^{i\theta}$$

The value $\theta$ is called an *eigenphase* that corresponds to this eigenvector.

> How can you prove that the modulus of an eigenvalue of a unitary matrix equals $1$?
> 
> On one hand, by definition of an eigenvalue, 
> $$U \ket{v} = \lambda \ket{v}$$
> $$|U \ket{v}| = |\lambda| \cdot |\ket{v}|$$
> On the other hand, using the properties of the unitary matrix, you can write the following equation:
> $$|U \ket{v}|^2 = \bra{v} U^\dagger U \ket{v} = \bra{v} U^{-1} U \ket{v} = \bra{v} I \ket{v} = \braket{v|v} = |\ket{v}|^2$$
>
> From these two equations, you get the following equality:
> $$(|\lambda| \cdot |\ket{v}|)^2 = |\ket{v}|^2$$
> And then, finally:
> $$|\lambda| = 1$$

If the quantum gate is self-adjoint, that is, its matrix equals its inverse $U^{-1} = U$, the eigenvalues of this matrix can only be $+1$ and $-1$, with eigenphases $0$ and $\pi$, respectively.

> You can prove this in a similar manner, using the defintion of an eigenvalue:
> $$U^2 \ket{v} = U(U \ket{v}) = U(\lambda \ket{v}) = \lambda U \ket{v} = \lambda^2 \ket{v}$$
> At the same time,
> $$U^2 \ket{v} = UU \ket{v} = U^{-1}U \ket{v} = I \ket{v} = \ket{v}$$
> So you can conclude that $\lambda^2 = 1$.

For example, the $Z$ gate has two eigenvctors:
- $\ket{0}$, with eigenvalue $1$
- $\ket{1}$, with eigenvalue $-1$


@[exercise]({
    "id": "phase_estimation__eigenvalues_s",
    "title": "Find Eigenvalues of the S Gate",
    "path": "./eigenvalues_s/"
})

@[exercise]({
    "id": "phase_estimation__eigenvectors_x",
    "title": "Find Eigenvectors of the X Gate",
    "path": "./eigenvectors_x/"
})

@[exercise]({
    "id": "phase_estimation__state_eigenvector",
    "title": "Is Given State an Eigenvector of the Gate?",
    "path": "./state_eigenvector/"
})


@[section]({
    "id": "phase_estimation__problem",
    "title": "Phase Estimation Problem"
})

The phase estimation problem is formulated as follows. 

You are given a unitary operator $U$ and its eigenvector $\ket{\psi}$. The eigenvector is given as a unitary operator $P$ that, when applied to $\ket{0}$, results in the state $\ket{\psi}$.

Your goal is to find the eigenvalue $\lambda$ associated with this eigenvector, or, in a more common formulation, the corresponding eigenphase $\theta$:

$$U\ket{\psi} = e^{2 \pi i \theta} \ket{\psi}, \theta = ?$$

The value of $\theta$ is defined to be between $0$ and $1$, since any value outside of this range has an equivalent value within it. Instead of representing $\theta$ as a decimal, sometimes it is represented as a binary fraction with $n$ digits:

$$\theta = 0.\theta_1 \theta_2... \theta_n = \frac{\theta_1}{2^1}+ \frac{\theta_2}{2^2}+...\frac{\theta_n}{2^n}$$

Let's consider a simplified variant of the phase estimation problem, in which you are guaranteed that the phase $\theta$ has exactly one binary digit, that is, it's either $0$ or $\frac12$.

@[exercise]({
    "id": "phase_estimation__one_bit_eigenphase",
    "title": "One-Bit Phase Estimation",
    "path": "./one_bit_eigenphase/"
})


@[section]({
    "id": "phase_estimation__qpe",
    "title": "Quantum Phase Estimation Algorithm"
})

- theory
- exercise: task 1.4 to implement QPE
- demo of end-to-end probabilistic behavior in case of lower precision (use R1 gate)


@[section]({
    "id": "phase_estimation__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned about the phase estimation problem and its solution using the quantum phase estimation algorithm.

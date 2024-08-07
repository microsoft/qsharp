The classical Fourier transform produces a time-domain signal's decomposition in the frequency domain, and the inverse Fourier transform produces a frequency-domain signal's time-domain representation.

Keeping this in mind, if you have the state that is the result of applying QFT to a basis state $\ket{F}$, the value $F$ can be recovered by applying the inverse QFT to this state and then measuring the qubits to find the resulting basis state.

Keep in mind that QFT acts on arrays in big endian notation, and `MeasureInteger` assumes that the input is in little endian, so you need to reverse the register before measuring it and coverting results to an integer.

@[solution]({
    "id": "qft__signal_frequency_solution",
    "codePath": "./Solution.qs"
})

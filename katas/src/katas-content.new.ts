export const learning = {
  codeDependencies: [
    {
      id: "library__katas",
      source: "",
    },
    {
      id: "qubit__common",
      source: "",
    },
    {
      id: "single_qubit_gates__common",
      source: "",
    },
  ],
  katas: [
    {
      id: "qubit",
      title: "The Qubit",
      sections: [
        {
          type: "text",
          id: "qubit__text__000",
          contentAsMarkdown:
            "# The Qubit\n" +
            "\n" +
            "This tutorial introduces you to one of the core concepts in quantum computing - the qubit, and its representation in mathematical notation and in Q# code.\n" +
            "\n" +
            "This tutorial assumes familiarity with complex arithmetic and linear algebra.\n" +
            "\n" +
            "This tutorial covers the following topics:\n" +
            "\n" +
            "- The concept of a qubit\n" +
            "- Superposition\n" +
            "- Vector representation of qubit states\n" +
            "- Dirac notation\n",
        },
        {
          type: "example",
          id: "qubit__qubit_data_type",
          title: "Qubit data type",
          code:
            "namespace Kata {\n" +
            "    open Microsoft.Quantum.Diagnostics;\n" +
            "    open Microsoft.Quantum.Intrinsic;\n" +
            "\n" +
            "    @EntryPoint()\n" +
            "    operation RunExample() : Unit {\n" +
            "        // This line allocates a qubit in state |0⟩\n" +
            "        use q = Qubit();\n" +
            '        Message("State |0⟩:");\n' +
            "\n" +
            "        // This line prints out the state of the quantum computer\n" +
            "        // Since only one qubit is allocated, only its state is printed\n" +
            "        DumpMachine();\n" +
            "\n" +
            "        // This line changes the qubit from state |0⟩ to state |1⟩\n" +
            "        X(q);\n" +
            "\n" +
            '        Message("State |1⟩:");\n' +
            "        DumpMachine();\n" +
            "\n" +
            "        // This line changes the qubit to state |-⟩ = (1/sqrt(2))(|0⟩ - |1⟩)\n" +
            "        // That is, this puts the qubit into a superposition\n" +
            "        // 1/sqrt(2) is approximately 0.707107\n" +
            "        H(q);\n" +
            "\n" +
            '        Message("State |-⟩:");\n' +
            "        DumpMachine();\n" +
            "\n" +
            "        // This line changes the qubit to state |-i⟩ = (1/sqrt(2))(|0⟩ - i|1⟩)\n" +
            "        S(q);\n" +
            "\n" +
            '        Message("State |-i⟩:");\n' +
            "        DumpMachine();\n" +
            "\n" +
            "        // This will put the qubit into an uneven superposition,\n" +
            "        // where the amplitudes of |0⟩ and |1⟩ have different moduli\n" +
            "        Rx(2.0, q);\n" +
            "        Ry(1.0, q);\n" +
            "\n" +
            '        Message("Uneven superposition state:");\n' +
            "        DumpMachine();\n" +
            "\n" +
            "        // This line returns the qubit to state |0⟩\n" +
            "        Reset(q);\n" +
            "    }\n" +
            "}",
          contentAsMarkdown:
            "# Qubit data type\n" +
            "\n" +
            "In Q#, qubits are represented by the `Qubit` data type. On a physical quantum computer, it's impossible to directly access the state of a qubit, whether to read its exact state, or to set it to a desired state, and this data type reflects that. Instead, you can change the state of a qubit using quantum gates, and extract information about the state of the system using measurements.\n" +
            "\n" +
            "That being said, when you run Q# code on a quantum simulator instead of a physical quantum computer, you can use diagnostic functions that allow you to peek at the state of the quantum system. This is very useful both for learning and for debugging small Q# programs.\n" +
            "\n" +
            `The qubits aren't an ordinary data type, so the variables of this type have to be declared and initialized ("allocated") a little differently:\n` +
            "\n" +
            "Freshly allocated qubits start out in state $|0\\rangle$, and have to be returned to that state by the time they are released. If you attempt to release a qubit in any state other than $|0\\rangle$ will result in a runtime error. We will see why it is important later, when we look at multi-qubit systems.\n",
        },
        {
          type: "text",
          id: "qubit__text__001",
          title: "Relative and Global Phase",
          contentAsMarkdown:
            "# Relative and Global Phase\n" +
            "\n" +
            "You may recall that a complex number has a parameter called its phase. If a complex number $x$ is written in polar form $x = re^{i\\theta}$, its phase is $\\theta$.\n" +
            "\n" +
            "The phase of a basis state is the complex phase of the amplitude of that state. For example, a system in state $\\frac{1 + i}{2}|0\\rangle + \\frac{1 - i}{2}|1\\rangle$, the phase of $|0\\rangle$ is $\\frac{\\pi}{4}$, and the phase of $|1\\rangle$ is $-\\frac{\\pi}{4}$. The difference between these two phases is known as **relative phase**.\n" +
            "\n" +
            "Multiplying the state of the entire system by $e^{i\\theta}$ doesn't affect the relative phase: $\\alpha|0\\rangle + \\beta|1\\rangle$ has the same relative phase as $e^{i\\theta}\\big(\\alpha|0\\rangle + \\beta|1\\rangle\\big)$. In the second expression, $\\theta$ is known as the system's **global phase**.\n" +
            "\n" +
            "The state of a qubit (or, more generally, the state of a quantum system) is defined by its relative phase - global phase arises as a consequence of using linear algebra to represent qubits, and has no physical meaning. That is, applying a phase to the entire state of a system (multiplying the entire vector by $e^{i\\theta}$ for any real $\\theta$) doesn't actually affect the state of the system. Because of this, global phase is sometimes known as **unobservable phase** or **hidden phase**.\n",
          contentAsHtml:
            '<h1 id="relative-and-global-phase">Relative and Global Phase</h1>\n' +
            "<p>You may recall that a complex number has a parameter called its phase. If a complex number $x$ is written in polar form $x = re^{i\\theta}$, its phase is $\\theta$.</p>\n" +
            "<p>The phase of a basis state is the complex phase of the amplitude of that state. For example, a system in state $\\frac{1 + i}{2}|0\\rangle + \\frac{1 - i}{2}|1\\rangle$, the phase of $|0\\rangle$ is $\\frac{\\pi}{4}$, and the phase of $|1\\rangle$ is $-\\frac{\\pi}{4}$. The difference between these two phases is known as <strong>relative phase</strong>.</p>\n" +
            "<p>Multiplying the state of the entire system by $e^{i\\theta}$ doesn&#39;t affect the relative phase: $\\alpha|0\\rangle + \\beta|1\\rangle$ has the same relative phase as $e^{i\\theta}\\big(\\alpha|0\\rangle + \\beta|1\\rangle\\big)$. In the second expression, $\\theta$ is known as the system&#39;s <strong>global phase</strong>.</p>\n" +
            "<p>The state of a qubit (or, more generally, the state of a quantum system) is defined by its relative phase - global phase arises as a consequence of using linear algebra to represent qubits, and has no physical meaning. That is, applying a phase to the entire state of a system (multiplying the entire vector by $e^{i\\theta}$ for any real $\\theta$) doesn&#39;t actually affect the state of the system. Because of this, global phase is sometimes known as <strong>unobservable phase</strong> or <strong>hidden phase</strong>.</p>\n",
        },
        {
          type: "exercise",
          id: "single_qubit_gates__y_gate",
          contentAsMarkdown:
            "### The $Y$ gate\n" +
            "\n" +
            "**Input:** A qubit in an arbitrary state $|\\\\psi\\\\rangle = \\\\alpha|0\\\\rangle + \\\\beta|1\\\\rangle$.\n" +
            "\n" +
            "**Goal:** Apply the Y gate to the qubit, i.e., transform the given state into $i\\\\alpha|1\\\\rangle - i\\\\beta|0\\\\rangle$.\n",
          contentAsHtml:
            '<h3 id="the-y-gate">The $Y$ gate</h3>\n' +
            "<p><strong>Input:</strong> A qubit in an arbitrary state $|\\psi\\rangle = \\alpha|0\\rangle + \\beta|1\\rangle$.</p>\n" +
            "<p><strong>Goal:</strong> Apply the Y gate to the qubit, i.e., transform the given state into $i\\alpha|1\\rangle - i\\beta|0\\rangle$.</p>\n",
          codeDependencies: ["library__katas", "qubit__common"],
          verificationCode:
            "namespace Kata {\n" +
            "    open Microsoft.Quantum.Intrinsic;\n" +
            "    open Microsoft.Quantum.Katas;\n" +
            "\n" +
            "    operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {\n" +
            "        Y(q);\n" +
            "    }\n" +
            "\n" +
            "    operation VerifyExercise() : Bool {\n" +
            "        let isCorrect = VerifySingleQubitOperation(ApplyY, ApplyYReference);\n" +
            "\n" +
            "        // Output different feedback to the user depending on whether the exercise was correct.\n" +
            "        use target = Qubit[1];\n" +
            "        let op = register => ApplyY(register[0]);\n" +
            "        let reference = register => ApplyYReference(register[0]);\n" +
            "        if isCorrect {\n" +
            "            ShowEffectOnQuantumState(target, op);\n" +
            "        } else {\n" +
            "            ShowQuantumStateComparison(target, op, reference);\n" +
            "        }\n" +
            "\n" +
            "        isCorrect\n" +
            "    }\n" +
            "}",
          placeholderCode:
            "namespace Kata {\n" +
            "    operation ApplyY(q : Qubit) : Unit is Adj + Ctl {\n" +
            "        // ...\n" +
            "\n" +
            "    }\n" +
            "}",
          solutionCode:
            "namespace Kata {\n" +
            "    operation ApplyY(q : Qubit) : Unit is Adj + Ctl {\n" +
            "        // Apply the Pauli Y operation.\n" +
            "        Y(q);\n" +
            "    }\n" +
            "}",
          solutionAsMarkdown: "## Solution\n",
          solutionAsHtml: '<h2 id="solution">Solution</h2>\n',
        },
      ],
    },
  ],
};

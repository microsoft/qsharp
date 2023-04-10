export type KatasMetadata = {
    modules: Array<{
        id: string;
        title: string;
        exercises: Array<{
            id: string;
            title: string;
            sourcePath : string;
        }>;
    }>;
};

export const katasMetadata: KatasMetadata = {
    modules: [
        {
            id: "single-qubit-gates",
            title: "Single-Qubit Gates",
            exercises: [
                {
                    id: "single-qubit-gates_y-gate",
                    title: "The Y Gate",
                    sourcePath: "single_qubit_gates/y_gate"
                },
                {
                    id: "single-qubit-gates_global-phase-i",
                    title: "Global Phase i",
                    sourcePath: "single_qubit_gates/global_phase_i"
                }
            ]
        },
        {
            id: "multi-qubit-gates",
            title: "Multi-Qubit Gates",
            exercises: []
        }
    ]
}
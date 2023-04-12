export type KatasMetadata = {
    modules: Array<{
        id: string;
        title: string;
        directory: string;
        exercises: Array<{
            id: string;
            title: string;
            directory : string;
        }>;
    }>;
};

export const katasMetadata: KatasMetadata = {
    modules: [
        {
            id: "single-qubit-gates",
            title: "Single-Qubit Gates",
            directory: "single_qubit_gates",
            exercises: [
                {
                    id: "single-qubit-gates_y-gate",
                    title: "The Y Gate",
                    directory: "y_gate"
                },
                {
                    id: "single-qubit-gates_global-phase-i",
                    title: "Global Phase i",
                    directory: "global_phase_i"
                }
            ]
        },
        {
            id: "multi-qubit-gates",
            title: "Multi-Qubit Gates",
            directory: "multi_qubit_gates",
            exercises: []
        }
    ]
}
//@ts-check

/**
 * Katas Organization
 * 
 * A Kata is a top-level container of educational items (exercises and examples) which are used to explain a particular
 * topic.
 * 
 * This file contains the information needed to build the content for all the Katas, and its main purpose is to convey
 * ordering. Each Kata and each item within a Kata  is listed in the order it is meant to be presented to students.
 * 
 * Each Kata is organized in a directory where a content.md file is present, and multiple sub-directories. Each
 * sub-directory represents an item within the Kata and its specific content depends on the type of item it represents.
 */

/**
 * @type {Array<
 * {
 *      directory: string,
 *      items: Array<{
 *          type: string,
 *          directory: string
 *      }>
 * }>}
 */
exports.katas = [
    {
        directory: "single_qubit_gates",
        items: [
            {
                type: "exercise",
                directory: "y_gate"
            },
            {
                type: "exercise",
                directory: "global_phase_i"
            }
        ]
    },
    {
        directory: "multi_qubit_gates",
        items: [
            {
                type: "exercise",
                directory: "preparing_bell_state"
            }
        ]
    }
];

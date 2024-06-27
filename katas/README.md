# Katas

A Kata is a top-level container of educational items which are used to explain a particular quantum computing topic using Q#. They are organized in sections which can be of three types:

- Lessons: text content that sometimes contains Q# code examples.
- Exercises: problems that the user solves by writting Q# code.
- Questions: analytical problems that have a text answer.

## Run Katas Online

Visit [Learn with Azure Quantum katas](https://quantum.microsoft.com/experience/quantum-katas) to try the new online Azure Quantum katas experience, with integrated assistance from Copilot in Azure Quantum.

## Build Katas Locally

We need to build the `playground` module to see the built katas locally. For the detailed instructions, refer to [Building Playground Locally](../playground/README.md#building-the-playground-locally).

## Rust crate

The katas crate exposes an API to check solutions for exercises.

## Composition

Katas are composed through an index.md markdown file that uses macros to indicate how to produce content for different kinds of sections.

### Macros

Macros are meant to insert interactive elements into the content defined through markdown. A macro starts with the `@` character followed by a word inside square brackets where the word represents the name of the macro (e.g. `@[example]`). This is followed by a JSON string enclosed within parenthesis `({...})` where the JSON string represents the properties of the macro that determine the interactive content. The macro is terminated by a line break `\r?\n`.

The following macros are available for katas composition:
- @[exercise]: Used to create Q# code exercises that we can be automatically verified.
    - id: Unique identifier for the exercise.
    - title: Title that will be displayed for the exercise.
    - path: Path to a folder that contains the description of the exercise. This folder should contain the following files:
        - `index.md`: the Markdown description of the exercise.
        - `Placeholder.qs`: the Q# code that is given to the learner to start with.
        - `Verification.qs`: the Q# code that checks whether the learner's solution is correct.
        - `solution.md`: the Markdown description of the solution(s) to the exercise.
        - `Solution.qs`: the Q# code that contains a "reference solution" described in `solution.md`.
    The @EntryPoint operation is called to check the solution (eventually, for the convention is to call Kata.Verification.CheckSolution).
- @[example]: Standalone Q# code snippets that can be referenced from markdown files.
    - id: Unique identifier for the example.
    - codePath: Path to a Q# file that contains the example code.
- @[solution]: represents a solution to a Q# code exercise. It is meant to be compiled as if it was the user authored code that solves a Q# code exercise. It can only be used in solution markdown files.
    - id: Unique identifier for the solution.
    - codePath: Path to a Q# file that contains the solution code.
- @[section]: A kata is broken into multiple sections. This starts a new section. Exercises are their own sections.
    - id: Unique identifier for the section.
    - title: Title of the section.

## Acknowledgements

Content of these web-based katas is largely a port of the previous effort located in the [QuantumKatas](https://github.com/microsoft/QuantumKatas) repository. Please refer to that repository for a history of contributions.

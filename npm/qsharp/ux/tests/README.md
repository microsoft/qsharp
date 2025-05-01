# Circuit Component UI Testing

This directory contains UI tests for the Circuit component using Playwright. These tests validate that the Circuit component renders correctly and responds appropriately to user interactions.

## Test Structure

The tests are organized into three main categories:

1. **Basic Component Tests** (`circuit.test.ts`): Tests the basic rendering and functionality of the Circuit component, such as correct DOM structure, rendering different circuit types, and zoom controls.

2. **Circuit Editor Tests** (`circuit-editor.test.ts`): Tests the interactive editing capabilities of the Circuit component, including adding/removing qubits and ensuring callbacks work correctly.

3. **Layout Tests** (`circuit-layout.test.ts`): Tests specific layout aspects of the Circuit component to ensure proper spatial relationships between circuit elements. This verifies that gates, lines, and controls are positioned correctly relative to each other.

## Test Data

The test circuits used by the tests are defined in `test-circuits.ts`. These include:

- Bell pair circuit (common entanglement example)
- Parameterized circuit (with rotation gates and parameters)
- Large circuit (for testing rendering limits)
- Multi-circuit group (with multiple circuits)

## Running the Tests

You can run the tests using the following npm commands from the `/workspaces/qsharp/npm/qsharp` directory:

```bash
# Run all UI tests
npm run test:ui

# Run UI tests in debug mode with visual browser
npm run test:ui:debug
```

## Test Page

The tests use a dedicated HTML page (`test-pages/circuit-test.html`) that provides a controlled environment for rendering the Circuit component during tests. The page is served using an Express server defined in `serve-test-page.js`.

## Adding New Tests

When adding new tests:

1. If needed, add new test circuits to `test-circuits.ts`
2. Add the test to the appropriate test file based on what aspect you're testing
3. Follow the existing patterns for injecting circuits and evaluating results

## Requirements

These tests require:

- Node.js
- Playwright with Chromium installed
- Express (for the test server)

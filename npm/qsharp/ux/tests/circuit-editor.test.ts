import { test, expect } from "@playwright/test";
import { getBellPairCircuit } from "./test-circuits";

// An empty circuit with two qubits - perfect for testing adding new operations
const emptyCircuit = {
  version: 1,
  circuits: [
    {
      qubits: [{ id: 0 }, { id: 1 }],
      componentGrid: [],
    },
  ],
};

test.describe("Circuit Editor Tests", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the test page
    await page.goto("/circuit-test.html");
  });

  test("edit callback is triggered when circuit is modified", async ({
    page,
  }) => {
    // Setup a variable to hold the callback data
    await page.evaluate(() => {
      window.lastEditCallbackData = null;
    });

    // Inject the circuit data and render the component
    await page.evaluate((circuitData) => {
      const container = document.getElementById("test-container");

      // Load required modules dynamically
      const script = document.createElement("script");
      script.type = "module";
      script.textContent = `
        import { render } from 'https://esm.sh/preact@10.20.0/compat';
        import { Circuit } from '/ux/circuit.js';

        // Render the Circuit component with an edit callback
        render(
          Circuit({
            circuit: ${JSON.stringify(circuitData)},
            isEditable: true,
            editCallback: (fileData) => {
              window.lastEditCallbackData = fileData;
            }
          }),
          document.getElementById('test-container')
        );
      `;
      document.body.appendChild(script);
    }, emptyCircuit);

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // Verify that add/remove qubit buttons exist
    const addQubitButton = await page.locator(".add-qubit-line");
    await expect(addQubitButton).toBeVisible();

    // Click the add qubit button to add a new qubit
    await addQubitButton.click();

    // Verify that the callback was called by checking the window variable
    const callbackData = await page.evaluate(() => window.lastEditCallbackData);
    expect(callbackData).not.toBeNull();
    expect(callbackData.circuits[0].qubits.length).toBe(3); // Should now have 3 qubits
  });

  test("editable circuit shows the panel and toolbox", async ({ page }) => {
    // Inject the circuit data and render the component
    await page.evaluate((circuitData) => {
      const container = document.getElementById("test-container");

      // Load required modules dynamically
      const script = document.createElement("script");
      script.type = "module";
      script.textContent = `
        import { render } from 'https://esm.sh/preact@10.20.0/compat';
        import { Circuit } from '/ux/circuit.js';

        // Render the Circuit component
        render(
          Circuit({
            circuit: ${JSON.stringify(circuitData)},
            isEditable: true
          }),
          document.getElementById('test-container')
        );
      `;
      document.body.appendChild(script);
    }, getBellPairCircuit());

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // No zoom control in editable mode
    const zoomControl = await page.locator("#qs-circuit-zoom").count();
    expect(zoomControl).toBe(0);

    // Verify that the panel exists
    const panel = await page.locator(".panel");
    await expect(panel).toBeVisible();

    // Verify qubit line controls
    const qubitLineControl = await page.locator(".qubit-line-control");
    await expect(qubitLineControl).toBeVisible();

    // Verify toolbox gates exist
    const gates = await page.locator(".panel .gate");
    expect(await gates.count()).toBeGreaterThan(5); // Should have multiple gates

    // Check for specific gates in the toolbox
    const gateTypes = ["H", "X", "Z", "SWAP", "RX", "RY", "RZ"];
    for (const gateType of gateTypes) {
      const gateInToolbox = await page.locator(
        `.panel .gate:has-text("${gateType}")`,
      );
      await expect(gateInToolbox).toBeVisible();
    }
  });

  test("can remove a qubit line", async ({ page }) => {
    // Setup a variable to hold the callback data
    await page.evaluate(() => {
      window.lastEditCallbackData = null;
    });

    // Inject the circuit data and render the component
    await page.evaluate((circuitData) => {
      const container = document.getElementById("test-container");

      // Load required modules dynamically
      const script = document.createElement("script");
      script.type = "module";
      script.textContent = `
        import { render } from 'https://esm.sh/preact@10.20.0/compat';
        import { Circuit } from '/ux/circuit.js';

        // Render the Circuit component with an edit callback
        render(
          Circuit({
            circuit: ${JSON.stringify(circuitData)},
            isEditable: true,
            editCallback: (fileData) => {
              window.lastEditCallbackData = fileData;
            }
          }),
          document.getElementById('test-container')
        );
      `;
      document.body.appendChild(script);
    }, emptyCircuit);

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // Check initial qubit count
    let qubitLines = await page.locator(".qs-circuit svg .qubit-line");
    expect(await qubitLines.count()).toBe(2);

    // Find the remove qubit button
    const removeQubitButton = await page.locator(".remove-qubit-line");
    await expect(removeQubitButton).toBeVisible();

    // Click the remove qubit button
    await removeQubitButton.click();

    // Verify the qubit was removed
    qubitLines = await page.locator(".qs-circuit svg .qubit-line");
    expect(await qubitLines.count()).toBe(1);

    // Verify that the callback was called
    const callbackData = await page.evaluate(() => window.lastEditCallbackData);
    expect(callbackData).not.toBeNull();
    expect(callbackData.circuits[0].qubits.length).toBe(1);
  });
});

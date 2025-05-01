import { test, expect } from "@playwright/test";
import {
  getBellPairCircuit,
  getParameterizedCircuit,
  getLargeCircuit,
} from "./test-circuits";

test.describe("Circuit Component Tests", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the test page
    await page.goto("/circuit-test.html");
  });

  test("renders a Bell pair circuit correctly", async ({ page }) => {
    // Inject the circuit data and render the component
    await page.evaluate((circuitData) => {
      const container = document.getElementById("test-container");

      // Load required modules dynamically
      const script = document.createElement("script");
      script.type = "module";
      script.textContent = `
        import { render } from 'https://esm.sh/preact@10.20.0/compat';
        import { Circuit } from '/ux/circuit.js';
        import * as qviz from '/ux/circuit-vis/index.js';

        // Render the Circuit component
        render(
          Circuit({
            circuit: ${JSON.stringify(circuitData)},
            isEditable: false
          }),
          document.getElementById('test-container')
        );
      `;
      document.body.appendChild(script);
    }, getBellPairCircuit());

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // Verify the SVG is created
    const svg = await page.locator(".qs-circuit svg");
    await expect(svg).toBeVisible();

    // Verify specific elements are in the DOM
    await expect(page.locator(".qs-circuit svg .qubit-line")).toHaveCount(2);

    // Check for gate elements
    const gateElementsCount = await page
      .locator(".qs-circuit svg .gate")
      .count();
    expect(gateElementsCount).toBeGreaterThan(0);

    // Verify H gate exists
    const hGate = await page.locator('.qs-circuit svg text:has-text("H")');
    await expect(hGate).toBeVisible();

    // Verify controlled-X gate connection
    const controlLine = await page.locator(".qs-circuit svg .control-line");
    await expect(controlLine).toBeVisible();

    // Verify measurement exists
    const measurement = await page.locator(".qs-circuit svg .measurement");
    await expect(measurement).toBeVisible();
  });

  test("renders a parameterized circuit correctly", async ({ page }) => {
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
            isEditable: false
          }),
          document.getElementById('test-container')
        );
      `;
      document.body.appendChild(script);
    }, getParameterizedCircuit());

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // Verify RX gate exists with parameter
    const rxGate = await page.locator('.qs-circuit svg text:has-text("RX")');
    await expect(rxGate).toBeVisible();

    // Verify parameter value (theta) appears
    const thetaParam = await page.locator(
      '.qs-circuit svg text:has-text("1.5708")',
    );
    await expect(thetaParam).toBeVisible();
  });

  test("shows zoom controls for non-editable circuits", async ({ page }) => {
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
            isEditable: false
          }),
          document.getElementById('test-container')
        );
      `;
      document.body.appendChild(script);
    }, getBellPairCircuit());

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // Verify zoom control is present
    const zoomControl = await page.locator("#qs-circuit-zoom");
    await expect(zoomControl).toBeVisible();

    // Test zoom functionality
    await zoomControl.fill("50");
    await page.keyboard.press("Enter");

    // Verify that the SVG width changed after zooming
    const svg = await page.locator(".qs-circuit svg");
    const style = await svg.getAttribute("style");
    expect(style).toContain("width:");
    expect(style).not.toContain("width: 100%");
  });

  test("shows error for too many qubits", async ({ page }) => {
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
            isEditable: false
          }),
          document.getElementById('test-container')
        );
      `;
      document.body.appendChild(script);
    }, getLargeCircuit());

    // Wait for the error message to appear
    await page.waitForSelector(".qs-circuit-error");

    // Verify the error message content
    const errorText = await page.locator(".qs-circuit-error").innerText();
    expect(errorText).toContain("too many qubits");
    expect(errorText).toContain("maximum supported is 1000");
  });
});

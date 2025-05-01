import { test, expect } from "@playwright/test";
import { getBellPairCircuit, getParameterizedCircuit } from "./test-circuits";
import fs from "fs";

/**
 * These tests focus specifically on layout/positioning aspects
 * of the Circuit component rendering.
 */
test.describe("Circuit Layout Tests", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the test page
    await page.goto("/circuit-test.html");
  });

  test("qubit lines are properly spaced vertically", async ({ page }) => {
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

    // Save the current state of the DOM to a file for troubleshooting
    const domContent = await page.evaluate(
      () => document.documentElement.outerHTML,
    );
    fs.writeFileSync(
      "/workspaces/qsharp/npm/qsharp/ux/tests/debug-dom.html",
      domContent,
    );

    // Capture console output and write it to a file
    page.on("console", (msg) => {
      const logMessage = `[${msg.type()}] ${msg.text()}\n`;
      fs.appendFileSync(
        "/workspaces/qsharp/npm/qsharp/ux/tests/debug-console.log",
        logMessage,
      );
    });

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // Get all qubit lines
    const qubitLines = await page.locator(".qs-circuit svg .qubit-line").all();
    expect(qubitLines.length).toBeGreaterThan(1);

    // Get bounding client rect for each qubit line
    const qubitPositions = await Promise.all(
      qubitLines.map(async (line) => {
        return await line.boundingBox();
      }),
    );

    // Verify qubit lines are spaced apart vertically (not overlapping)
    for (let i = 0; i < qubitPositions.length - 1; i++) {
      const current = qubitPositions[i]!;
      const next = qubitPositions[i + 1]!;

      // The next qubit line should be positioned below the current one
      expect(next.y).toBeGreaterThan(current.y + current.height);

      // Lines should have consistent spacing
      if (i > 0) {
        const prevGap =
          qubitPositions[i]!.y -
          (qubitPositions[i - 1]!.y + qubitPositions[i - 1]!.height);
        const currentGap = next.y - (current.y + current.height);

        // Check if spacing is approximately equal (within 2 pixels)
        expect(Math.abs(prevGap - currentGap)).toBeLessThanOrEqual(2);
      }
    }
  });

  test("gates are centered on qubit lines", async ({ page }) => {
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

    // Get the H gate
    const hGate = await page.locator('.qs-circuit svg text:has-text("H")');
    const hGateBox = await hGate.boundingBox();

    // Get the first qubit line
    const qubitLine = await page.locator(".qs-circuit svg .qubit-line").first();
    const qubitLineBox = await qubitLine.boundingBox();

    // The H gate should be aligned with the first qubit line vertically
    // We'll check that the gate's vertical center is approximately aligned with the qubit line
    const hGateCenter = hGateBox!.y + hGateBox!.height / 2;
    const qubitLineCenter = qubitLineBox!.y + qubitLineBox!.height / 2;
    expect(Math.abs(hGateCenter - qubitLineCenter)).toBeLessThanOrEqual(10);
  });

  test("controlled gates have vertical lines connecting qubits", async ({
    page,
  }) => {
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

    // Find the control line (vertical connection between control and target qubits)
    const controlLine = await page.locator(".qs-circuit svg .control-line");
    await expect(controlLine).toBeVisible();

    // Get the bounding boxes for the control dot, control line, and target gate
    const controlDot = await page.locator(".qs-circuit svg .control-dot");
    const controlDotBox = await controlDot.boundingBox();

    const controlLineBox = await controlLine.boundingBox();

    const targetGate = await page.locator('.qs-circuit svg text:has-text("X")');
    const targetGateBox = await targetGate.boundingBox();

    // Verify the control line extends vertically from control to target
    // The line should have:
    // 1. Approximately same x-coordinate as the control dot
    expect(Math.abs(controlLineBox!.x - controlDotBox!.x)).toBeLessThanOrEqual(
      5,
    );

    // 2. Start near the control dot and extend downward past the target gate
    expect(controlLineBox!.y).toBeLessThanOrEqual(
      controlDotBox!.y + controlDotBox!.height,
    );
    expect(controlLineBox!.y + controlLineBox!.height).toBeGreaterThanOrEqual(
      targetGateBox!.y,
    );
  });

  test("operations in same column are horizontally aligned", async ({
    page,
  }) => {
    // Inject a circuit with operations in the same column
    const circuitWithColumn = {
      version: 1,
      circuits: [
        {
          qubits: [{ id: 0 }, { id: 1 }],
          componentGrid: [
            {
              components: [
                { kind: "unitary", gate: "H", targets: [{ qubit: 0 }] },
                { kind: "unitary", gate: "X", targets: [{ qubit: 1 }] },
              ],
            },
          ],
        },
      ],
    };

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
    }, circuitWithColumn);

    // Wait for the circuit to render
    await page.waitForSelector(".qs-circuit svg");

    // Get the H and X gates
    const hGate = await page.locator('.qs-circuit svg text:has-text("H")');
    const xGate = await page.locator('.qs-circuit svg text:has-text("X")');

    // Get their bounding boxes
    const hGateBox = await hGate.boundingBox();
    const xGateBox = await xGate.boundingBox();

    // The gates should be approximately horizontally aligned
    expect(Math.abs(hGateBox!.x - xGateBox!.x)).toBeLessThanOrEqual(10);
  });

  test("parameters appear correctly positioned with gates", async ({
    page,
  }) => {
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

    // Get the RX gate and its parameter
    const rxGate = await page.locator('.qs-circuit svg text:has-text("RX")');
    const paramValue = await page.locator(
      '.qs-circuit svg text:has-text("1.5708")',
    );

    // Get bounding boxes
    const rxGateBox = await rxGate.boundingBox();
    const paramBox = await paramValue.boundingBox();

    // The parameter should be positioned near the gate (usually below or to the right)
    // We'll check that it's either:
    // 1. Below the gate (y-coordinate greater than gate's bottom)
    // 2. To the right of the gate (x-coordinate greater than gate's right edge)
    const isBelow = paramBox!.y >= rxGateBox!.y + rxGateBox!.height;
    const isRight = paramBox!.x >= rxGateBox!.x + rxGateBox!.width;

    expect(isBelow || isRight).toBeTruthy();

    // The parameter should not be too far from the gate
    const horizontalDist = Math.abs(
      paramBox!.x + paramBox!.width / 2 - (rxGateBox!.x + rxGateBox!.width / 2),
    );
    const verticalDist = Math.abs(
      paramBox!.y +
        paramBox!.height / 2 -
        (rxGateBox!.y + rxGateBox!.height / 2),
    );

    // The parameter shouldn't be too far from its gate (within reasonable bounds)
    expect(horizontalDist + verticalDist).toBeLessThan(100);
  });
});

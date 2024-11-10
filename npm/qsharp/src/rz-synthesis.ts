// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/*
This file generates the rz-array.json file that contains the gates for Rz rotation synthesis.

Run with: npm run rz
*/

import { compare, Hadamard, Ident, M2x2, PauliX, TGate } from "./cplx.js";
import { writeFileSync } from "node:fs";

// An epsilon of 0.002 is about 500M tries to find 1256 points, mostly around 50 - 60 gates
const points = Math.floor(Math.PI * 2 * 200); // 1256 points around the sphere
const epsilon = 0.002;

let entries = 0;

if (points % 8 !== 0) throw "Subdivisions must be a multiple of 8";

const intervalCount = points / 8;
const intervalFound: Array<number> = new Array(intervalCount).fill(0);
let intervalFoundCount = 0;

// The power of 2 indicates how many operations.
// - Index 0 & 1 have 1 op.
// - Index 2, 3, 4, 5 have 2 ops.
// - Index 6, 7, 8, 9, 10, 11, 12, 13 have 3 ops.
// - etc.
// Math.log(x) returns 0 for 1, 1 for 2, 2 for 4, 3 for 8, etc.
const opCountForIndex = (x: number) => Math.floor(Math.log2(x + 2));
const startIndexForOpCount = (ops: number) => 2 ** ops - 2;

function indexToOpSequence(idx: number): string[] {
  // We'll use 'A' to indicate H then T, and 'B' to indicate H then T adjoint.

  const result: string[] = [];
  const ops = opCountForIndex(idx);
  for (let i = 0; i < ops; i++) {
    if (idx & 0x01) {
      result.push("A");
    } else {
      result.push("B");
    }
    idx >>= 1;
  }

  return result;
}

function opSequenceToGateSequence(seq: string[]): string {
  return seq
    .map((op) => (op === "A" ? "HT" : op === "B" ? "Ht" : op))
    .join("")
    .replace(/HH/g, "");
}

const Aseq = TGate.mul(Hadamard);
const Bseq = TGate.adjoint().mul(Hadamard);

const MatrixLenBytes = 4 * 2 * 4; // 4 bytes in a float, 2 floats in a complex, 4 complex values in a 2x2 matrix
const EntryCount = startIndexForOpCount(25); // Test up to 2^24 permutations

console.log(`Entries to pre-calculate: ${EntryCount}`);
console.log(`Using an espilon of ${epsilon}`);
console.log(`Intervals to find: ${intervalCount}`);

const entryBuffer = new ArrayBuffer(EntryCount * MatrixLenBytes);
const bufferView = new DataView(entryBuffer);

const phaseEntries: Array<{
  phase: number;
  sequence: string;
  matrix: M2x2;
  offset: number;
}> = [];

function onPhaseEntry(phase: number, ops: string[], matrix: M2x2) {
  const phaseIdx = Math.round((phase * points) / (2 * Math.PI));
  const segment = phaseIdx % intervalCount;
  if (intervalFound[segment] === 0) {
    intervalFound[segment] = 1;
    ++intervalFoundCount;
  }
  if (
    phaseEntries[segment] === undefined ||
    matrix.b.mag() * 1.25 < phaseEntries[segment].offset // 25% error improvement is worth extra gates
  ) {
    if (phaseEntries[segment] === undefined) ++entries;
    phaseEntries[segment] = {
      phase,
      sequence: opSequenceToGateSequence(ops),
      matrix,
      offset: matrix.b.mag(),
    };
    checkEntry(phaseEntries[segment]);
  }
}

function checkEntry(entry: { phase: number; sequence: string; matrix: M2x2 }) {
  let state = Ident.mul(1);
  for (const gate of entry.sequence) {
    if (gate === "H") {
      state = Hadamard.mul(state);
    } else if (gate === "X") {
      state = PauliX.mul(state);
    } else if (gate === "T") {
      state = TGate.mul(state);
    } else if (gate === "t") {
      state = TGate.adjoint().mul(state);
    }
  }
  if (!state.isDiagonal(epsilon)) {
    console.error(`offset error: `, entry);
  }
  if (!compare(state.phase(), entry.phase, epsilon)) {
    console.error(`Phase mismatch: ${state.phase()} != ${entry.phase}`, entry);
  }
}

function writeMatrixToBuffer(idx: number, matrix: M2x2) {
  const offset = idx * MatrixLenBytes;
  bufferView.setFloat32(offset + 0, matrix.a.re);
  bufferView.setFloat32(offset + 4, matrix.a.im);
  bufferView.setFloat32(offset + 8, matrix.b.re);
  bufferView.setFloat32(offset + 12, matrix.b.im);
  bufferView.setFloat32(offset + 16, matrix.c.re);
  bufferView.setFloat32(offset + 20, matrix.c.im);
  bufferView.setFloat32(offset + 24, matrix.d.re);
  bufferView.setFloat32(offset + 28, matrix.d.im);
}

function setMatrixFromBuffer(idx: number, matrix: M2x2) {
  const offset = idx * MatrixLenBytes;
  matrix.a.re = bufferView.getFloat32(offset + 0);
  matrix.a.im = bufferView.getFloat32(offset + 4);
  matrix.b.re = bufferView.getFloat32(offset + 8);
  matrix.b.im = bufferView.getFloat32(offset + 12);
  matrix.c.re = bufferView.getFloat32(offset + 16);
  matrix.c.im = bufferView.getFloat32(offset + 20);
  matrix.d.re = bufferView.getFloat32(offset + 24);
  matrix.d.im = bufferView.getFloat32(offset + 28);
}

function gatesList(gates: Array<string | number>): string[] {
  return gates
    .map((entry) => {
      if (typeof entry === "number") {
        return indexToOpSequence(entry);
      } else {
        return entry;
      }
    })
    .flat();
}

function checkMatrix(matrix: M2x2, gates: Array<string | number>) {
  // All matrices come in with a trailing T or t, but that's of no interest as without that gate it's also a phase operation
  //   if (matrix.isDiagonal(epsilon)) {
  //     onPhaseEntry(matrix.phase(), gatesList(gates), matrix);
  //   }

  const plusHEnd = Hadamard.mul(matrix);
  if (plusHEnd.isDiagonal(epsilon)) {
    onPhaseEntry(plusHEnd.phase(), gatesList([...gates, "H"]), plusHEnd);
  }
  // No point unshift the first H, else then it just starts with a phase shift (T or t)
  //   const plusHStart = matrix.mul(Hadamard);
  //   if (plusHStart.isDiagonal(epsilon)) {
  //     gates.unshift("H");
  //     onPhaseEntry(plusHStart.phase(), gatesList(gates), plusHStart);
  //   }
  const plusXEnd = PauliX.mul(matrix);
  if (plusXEnd.isDiagonal(epsilon)) {
    onPhaseEntry(plusXEnd.phase(), gatesList([...gates, "X"]), plusXEnd);
  }

  // Plus H & X is worth a check
  const plusHXEnd = PauliX.mul(plusHEnd);
  if (plusHXEnd.isDiagonal(epsilon)) {
    onPhaseEntry(plusHXEnd.phase(), gatesList([...gates, "H", "X"]), plusHXEnd);
  }
}

function fillBuffer(): boolean {
  // Fill the buffer up
  for (let i = 0; i < EntryCount; i++) {
    if (intervalFoundCount === intervalCount) return true;
    if (i % 1000000 === 0)
      console.log(
        `Processing entry ${i}. Intervals found: ${intervalFoundCount}. Gate length: ${opCountForIndex(i) * 2}`,
      );
    const gates = indexToOpSequence(i);
    const matrix = gates.reduce((acc, gate) => {
      if (gate === "A") {
        return Aseq.mul(acc);
      } else {
        return Bseq.mul(acc);
      }
    }, Ident.mul(1));
    writeMatrixToBuffer(i, matrix);
    checkMatrix(matrix, gates);
  }
  return false;
}

// Put 0 in
onPhaseEntry(0, [], Ident.mul(1));

if (!fillBuffer()) {
  // Just filling the buffer didn't find all the intervals, so we need to start running further permutations

  const startFullGatesIndex = startIndexForOpCount(24);
  const endFullGatesIndex = startIndexForOpCount(25) - 1;

  const fullMatrix = Ident.mul(1);
  const tinyMatrix = Ident.mul(1);

  let i = EntryCount;

  outer: for (let tinyIdx = 0; tinyIdx < startFullGatesIndex; tinyIdx++) {
    setMatrixFromBuffer(tinyIdx, tinyMatrix);
    for (
      let fullIdx = startFullGatesIndex;
      fullIdx <= endFullGatesIndex;
      fullIdx++
    ) {
      if (i % 1000000 === 0) {
        console.log(
          `Processing entry ${i}. Intervals found: ${intervalFoundCount}.  Gate length: ${(24 + opCountForIndex(tinyIdx)) * 2}`,
        );
      }
      ++i;
      if (intervalFoundCount === intervalCount) break outer;
      setMatrixFromBuffer(fullIdx, fullMatrix);
      const matrix = tinyMatrix.mul(fullMatrix);
      checkMatrix(matrix, [fullIdx, tinyIdx]);
    }
  }
}

const result = phaseEntries.map((entry) =>
  entry
    ? {
        phase: entry.phase,
        gates: entry.sequence,
        matrix: entry.matrix.toShortString(),
        offset: entry.matrix.b.mag(),
      }
    : null,
);

// Write a file with the details of each found rotation for manual inspection if needed
writeFileSync("./rz-details.json", JSON.stringify(result, null, 2), "utf8");

// Create the array with the index being the phase points, and the value being the gates
const phaseGates: string[] = new Array(points).fill("****");
phaseEntries.forEach((entry) => {
  const phaseIdx = Math.round((entry.phase * points) / (2 * Math.PI));
  phaseGates[phaseIdx] = entry.sequence;

  const Tidx = (phaseIdx + intervalCount) % points;
  phaseGates[Tidx] = entry.sequence + "T";

  const Sidx = (phaseIdx + intervalCount * 2) % points;
  phaseGates[Sidx] = entry.sequence + "S";

  const STidx = (phaseIdx + intervalCount * 3) % points;
  phaseGates[STidx] = entry.sequence + "ST";

  const Zidx = (phaseIdx + intervalCount * 4) % points;
  phaseGates[Zidx] = entry.sequence + "Z";

  const ZTidx = (phaseIdx + intervalCount * 5) % points;
  phaseGates[ZTidx] = entry.sequence + "ZT";

  const sidx = (phaseIdx + intervalCount * 6) % points;
  phaseGates[sidx] = entry.sequence + "s";

  const tidx = (phaseIdx + intervalCount * 7) % points;
  phaseGates[tidx] = entry.sequence + "t";
});

writeFileSync("./rz-array.json", JSON.stringify(phaseGates, null, 1), "utf8");

console.log(`Total entries: ${entries}`);

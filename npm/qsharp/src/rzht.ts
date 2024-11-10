import { Hadamard, Ident, M2x2, TGate, numToStr } from "./cplx.js";

const epsilon = 0.01;
let entries = 0;

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
    .map((op) => (op === "H" ? "H" : op === "A" ? "HT" : "HA"))
    .join("")
    .replace(/HH/g, "");
}

const Aseq = TGate.mul(Hadamard);
const Bseq = TGate.adjoint().mul(Hadamard);

const MatrixLenBytes = 4 * 2 * 4; // 4 bytes in a float, 2 floats in a complex, 4 complex values in a 2x2 matrix
// const EntryCount = startIndexForOpCount(23);
const EntryCount = startIndexForOpCount(25);
console.log(`Entires to calculate: ${EntryCount}`);
console.log(`Using an espilon of ${epsilon}`);

const entryBuffer = new ArrayBuffer(EntryCount * MatrixLenBytes);
const bufferView = new DataView(entryBuffer);

const phaseEntries: Array<{ phase: number; sequence: string; matrix: M2x2 }> =
  [];

function onPhaseEntry(phase: number, ops: string[], matrix: M2x2) {
  const phaseIdx = Math.round(phase * 100);
  if (phaseEntries[phaseIdx] === undefined) {
    ++entries;
    phaseEntries[phaseIdx] = {
      phase,
      sequence: opSequenceToGateSequence(ops),
      matrix,
    };
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

// Fill the buffer up
for (let i = 0; i < EntryCount; i++) {
  if (i % 1000000 === 0) console.log(`Processing entry ${i}`);
  const gates = indexToOpSequence(i);
  const matrix = gates.reduce((acc, gate) => {
    if (gate === "A") {
      return Aseq.mul(acc);
    } else {
      return Bseq.mul(acc);
    }
  }, Ident.mul(1));
  writeMatrixToBuffer(i, matrix);

  if (matrix.isDiagonal(epsilon)) {
    onPhaseEntry(matrix.phase(), gates, matrix);
  }

  const plusHEnd = Hadamard.mul(matrix);
  if (plusHEnd.isDiagonal(epsilon)) {
    gates.push("H");
    onPhaseEntry(plusHEnd.phase(), gates, plusHEnd);
  }
  const plusHStart = matrix.mul(Hadamard);
  if (plusHStart.isDiagonal(epsilon)) {
    gates.unshift("H");
    onPhaseEntry(plusHStart.phase(), gates, plusHStart);
  }
}

for (const entry in phaseEntries) {
  console.log(
    `Phase: ${numToStr(phaseEntries[entry].phase)}\nMatrix: ${phaseEntries[entry].matrix.toShortString()}\nGates: ${phaseEntries[entry].sequence}\n`,
  );
}

const result = phaseEntries.map((entry) =>
  entry
    ? {
        phase: entry.phase,
        gates: entry.sequence,
        matrix: entry.matrix.toShortString(),
      }
    : undefined,
);

console.log(JSON.stringify(result, null, 2));
console.log(`Total entries: ${entries}`);

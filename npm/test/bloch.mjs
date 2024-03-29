import assert from "node:assert";
import { describe, it } from "node:test";

import {
  Hadamard,
  PauliZ,
  PauliX,
  PauliY,
  Ident,
  SGate,
  TGate,
  m2x2,
  vec2,
  Cplx,
  Ket0,
  Ket1,
  KetPlus,
  KetMinus,
  KetPlusI,
  KetMinusI,
} from "../dist/cplx.js";

describe("Gate combos", () => {
  it("HZH† = X", () => {
    const HZHt = Hadamard.mul(PauliZ).mul(Hadamard.adjoint());
    assert(HZHt.compare(PauliX));
  });

  it("SS† = I", () => {
    const SSt = SGate.mul(SGate.adjoint());
    assert(SSt.compare(Ident));
  });

  it("TT = S", () => {
    const TT = TGate.mul(TGate);
    assert(TT.compare(SGate));
  });

  it("transposes", () => {
    const yTranspose = m2x2("0,i,-i,0");
    assert(PauliY.transpose().compare(yTranspose));
  });
});

describe("Gate application", () => {
  it("Applies Hadamard to |0>", () => {
    const result = Hadamard.mulVec2(Ket0);
    assert(result.compare(KetPlus));
  });

  it("Applies ZH to |0>", () => {
    const result = PauliZ.mulVec2(Hadamard.mulVec2(Ket0));

    assert(result.compare(KetMinus));
  });

  it("Applies XH to |0>", () => {
    const result = PauliX.mulVec2(Hadamard.mulVec2(Ket0));

    assert(result.compare(KetPlus));
  });

  it("Applies X to |0>", () => {
    const result = PauliX.mulVec2(Ket0);
    assert(result.compare(Ket1));
  });

  it("Applies Y to |0>", () => {
    const result = PauliY.mulVec2(Ket0);
    const expected = vec2("0,i");
    assert(result.compare(expected));
  });

  it("|0> lands in |+i> after Hadamard and SGate", () => {
    const Xplus = Hadamard.mulVec2(Ket0);
    const result = SGate.mulVec2(Xplus);
    assert(result.compare(KetPlusI));
  });

  it("|1> lands in |-i> after Hadamard and SGate", () => {
    const Xneg = Hadamard.mulVec2(Ket1);
    const result = SGate.mulVec2(Xneg);
    assert(result.compare(KetMinusI));
  });
});

describe("Math tests", () => {
  it("Checks tolerance inside bounds", () => {
    const a = new Cplx(1.0000002, 0);
    assert(Ident.a.compare(a));
  });

  it("Checks tolerance outside bounds", () => {
    const a = new Cplx(1.000002, 0);
    assert(!Ident.a.compare(a));
  });

  it("Checks matrix equality", () => {
    const mx = m2x2("1,0,0,1");
    assert(mx.compare(Ident));
  });

  it("Checks matrix inequality", () => {
    const mx = m2x2("1,0,0,i");
    assert(!mx.compare(Ident));
  });
});

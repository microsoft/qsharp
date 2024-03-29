// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Basic classes for complex numbers and 2x2 matrices/vectors

const epsilon = 0.000001; // Tolerance when comparing numbers
function compare(a: number, b: number): boolean {
  return Math.abs(a - b) < epsilon;
}

export class Cplx {
  constructor(
    public re: number,
    public im: number,
  ) {}

  static zero = new Cplx(0, 0);
  static one = new Cplx(1, 0);
  static i = new Cplx(0, 1);
  static negOne = new Cplx(-1, 0);
  static negI = new Cplx(0, -1);

  add(c: Cplx): Cplx {
    return new Cplx(this.re + c.re, this.im + c.im);
  }

  sub(c: Cplx): Cplx {
    return new Cplx(this.re - c.re, this.im - c.im);
  }

  mul(c: Cplx | number): Cplx {
    if (typeof c === "number") {
      return new Cplx(this.re * c, this.im * c);
    } else {
      // FOIL: (a + bi)(c + di) = ac + adi + bci + bdi^2
      return new Cplx(
        this.re * c.re - this.im * c.im,
        this.re * c.im + this.im * c.re,
      );
    }
  }

  mag(): number {
    return Math.sqrt(this.re ** 2 + this.im ** 2);
  }

  isUnit(): boolean {
    return Math.abs(this.mag() - 1.0) < epsilon;
  }

  norm(): Cplx {
    if (this.isUnit()) {
      return cplx(this);
    } else {
      return this.mul(1 / this.mag());
    }
  }

  conj(): Cplx {
    return new Cplx(this.re, -this.im);
  }

  compare(c: Cplx): boolean {
    return compare(this.re, c.re) && compare(this.im, c.im);
  }

  static parse(input: string): Cplx | null {
    // Valid formats: "0", "1", "i", "-i", "-1.2+3i", "-2i", etc.

    // Removal all whitespace
    input = input.replace(/\s/g, "");

    // Handle special and common cases
    switch (input) {
      case "0":
        return new Cplx(0, 0);
      case "1":
        return new Cplx(1, 0);
      case "i":
        return new Cplx(0, 1);
      case "-1":
        return new Cplx(-1, 0);
      case "-i":
        return new Cplx(0, -1);
      default:
        break;
    }

    // Regular expressions to parse a complex number
    // - Optional leading '-' sign
    // - Numeric value for real part (optional if imaginary part is present)
    // - Optional numeric imaginary part with leading [+-] and trailing 'i'
    //   - If real part is not present, leading '+' on imaginary part is optional
    // - Numeric parts are an integer followed by an optional decimal part
    // - If decimal part is present, it must contain at least one digit
    // - Scientific notation is not supported

    // To ease parsing, look for the real and imaginary parts separately
    const rePart = /^[-]?(\d+)(\.\d+)?(?=$|[+-])/;
    const imPart = /(^|[+-])(\d+)(\.\d+)?(?=i$)/;

    const reMatch = input.match(rePart);
    const imMatch = input.match(imPart);
    if (!reMatch && !imMatch) {
      return null;
    }

    const reVal = parseFloat(reMatch?.[0] ?? "0");
    const imVal = parseFloat(imMatch?.[0] ?? "0");
    return new Cplx(reVal, imVal);
  }

  toString() {
    const fmt = new Intl.NumberFormat("en-US", { maximumFractionDigits: 4 });
    const reTo4 = fmt.format(this.re);
    const imTo4 = compare(this.im, 1)
      ? ""
      : compare(this.im, -1)
        ? "-"
        : fmt.format(this.im);

    if (compare(this.im, 0)) {
      return reTo4;
    } else if (compare(this.re, 0)) {
      return `${imTo4}${"i"}`;
    } else {
      return `${reTo4}${this.im > 0 ? "+" : ""}${imTo4}i`;
    }
  }
}

export function cplx(x: number | string | Cplx): Cplx {
  if (typeof x === "number") {
    return new Cplx(x, 0);
  } else if (typeof x === "string") {
    const result = Cplx.parse(x);
    if (result) {
      return result;
    } else {
      throw Error("Invalid cplx string");
    }
  } else if (x instanceof Cplx) {
    return new Cplx(x.re, x.im);
  } else {
    throw Error("Invalid cplx parameter");
  }
}

export class Vec2 {
  constructor(
    public x: Cplx,
    public y: Cplx,
  ) {}
  add(v: Vec2): Vec2 {
    return new Vec2(this.x.add(v.x), this.y.add(v.y));
  }

  sub(v: Vec2): Vec2 {
    return new Vec2(this.x.sub(v.x), this.y.sub(v.y));
  }

  mul(v: number): Vec2 {
    return new Vec2(this.x.mul(v), this.y.mul(v));
  }

  innerProduct(v: Vec2): Cplx {
    return this.x.mul(v.x).add(this.y.mul(v.y));
  }

  outerProduct(v: Vec2): M2x2 {
    return new M2x2(
      this.x.mul(v.x),
      this.x.mul(v.y),
      this.y.mul(v.x),
      this.y.mul(v.y),
    );
  }

  compare(v: Vec2): boolean {
    return this.x.compare(v.x) && this.y.compare(v.y);
  }
}

export function vec2(input: Vec2 | string | (number | Cplx)[]): Vec2 {
  if (input instanceof Vec2) {
    return new Vec2(input.x, input.y);
  } else if (Array.isArray(input) && input.length === 2) {
    return new Vec2(cplx(input[0]), cplx(input[1]));
  } else if (typeof input === "string") {
    const parts = input.split(",");
    if (parts.length === 2) {
      return new Vec2(cplx(parts[0]), cplx(parts[1]));
    }
  }
  throw Error("Invalid input parameter to vec2");
}

// A 2x2 matrix of complex numbers
export class M2x2 {
  constructor(
    public a: Cplx,
    public b: Cplx,
    public c: Cplx,
    public d: Cplx,
  ) {}

  add(m: M2x2): M2x2 {
    return new M2x2(
      this.a.add(m.a),
      this.b.add(m.b),
      this.c.add(m.c),
      this.d.add(m.d),
    );
  }

  sub(m: M2x2): M2x2 {
    return new M2x2(
      this.a.sub(m.a),
      this.b.sub(m.b),
      this.c.sub(m.c),
      this.d.sub(m.d),
    );
  }

  mul(m: M2x2 | Cplx | number): M2x2 {
    if (typeof m === "number") m = cplx(m);
    if (m instanceof Cplx) {
      return new M2x2(
        this.a.mul(m),
        this.b.mul(m),
        this.c.mul(m),
        this.d.mul(m),
      );
    } else {
      return new M2x2(
        this.a.mul(m.a).add(this.b.mul(m.c)),
        this.a.mul(m.b).add(this.b.mul(m.d)),
        this.c.mul(m.a).add(this.d.mul(m.c)),
        this.c.mul(m.b).add(this.d.mul(m.d)),
      );
    }
  }

  mulVec2(v: Vec2): Vec2 {
    return new Vec2(
      this.a.mul(v.x).add(this.b.mul(v.y)),
      this.c.mul(v.x).add(this.d.mul(v.y)),
    );
  }

  compare(m: M2x2): boolean {
    return (
      this.a.compare(m.a) &&
      this.b.compare(m.b) &&
      this.c.compare(m.c) &&
      this.d.compare(m.d)
    );
  }

  det(): Cplx {
    return this.a.mul(this.d).sub(this.b.mul(this.c));
  }

  transpose(): M2x2 {
    return new M2x2(this.a, this.c, this.b, this.d);
  }

  adjoint(): M2x2 {
    return new M2x2(this.a.conj(), this.c.conj(), this.b.conj(), this.d.conj());
  }
}

export function m2x2(v: M2x2 | (number | Cplx)[] | string): M2x2 {
  if (v instanceof M2x2) {
    return new M2x2(v.a, v.b, v.c, v.d);
  } else if (Array.isArray(v) && v.length === 4) {
    return new M2x2(cplx(v[0]), cplx(v[1]), cplx(v[2]), cplx(v[3]));
  } else if (typeof v === "string") {
    const parts = v.split(",");
    if (parts.length === 4) {
      return new M2x2(
        cplx(parts[0]),
        cplx(parts[1]),
        cplx(parts[2]),
        cplx(parts[3]),
      );
    }
  }
  throw Error("Invalid input paramter to m2x2");
}

export function e_to_ix(x: number): Cplx {
  return new Cplx(Math.cos(x), Math.sin(x));
}

export const e_to_ipi_4 = new Cplx(Math.SQRT1_2, Math.SQRT1_2);

export const Ident = m2x2(`
  1, 0,
  0, 1
`);

export const PauliX = m2x2(`
  0, 1,
  1, 0
`);

export const PauliY = m2x2(`
  0,-i,
  i, 0
`);

export const PauliZ = m2x2(`
  1, 0,
  0,-1
`);

export const Hadamard = m2x2(`
  1, 1,
  1,-1
`).mul(Math.SQRT1_2);

export const SGate = m2x2(`
  1, 0,
  0, i
`);

export const TGate = m2x2([1, 0, 0, e_to_ipi_4]);

export const Ket0 = vec2([1, 0]);
export const Ket1 = vec2([0, 1]);
export const KetPlus = vec2([1, 1]).mul(Math.SQRT1_2);
export const KetMinus = vec2([1, -1]).mul(Math.SQRT1_2);
export const KetPlusI = vec2("1,i").mul(Math.SQRT1_2);
export const KetMinusI = vec2("1,-i").mul(Math.SQRT1_2);

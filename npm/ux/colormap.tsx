// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function ColorMap(colorsCount: number): string[] {
  const colors: string[] = [];
  const saturationIncrement = 1.4142135623730951; // square root of 2
  const hueIncrement = 0.618033988749895; // golden ratio
  const lightnessIncrement = 2.718281828459045; // euler's number

  for (let i = 0; i < colorsCount; i++) {
    const hue = (i * hueIncrement) % 1;
    const saturation = (1 - ((i * saturationIncrement) % 1)) * 0.5 + 0.5;
    const lightness = ((i * lightnessIncrement) % 1) * 0.3 + 0.35;
    const rgbColor = hslToRgb(hue, saturation, lightness);
    const hexColor = rgbToHex(rgbColor[0], rgbColor[1], rgbColor[2]);
    colors.push(hexColor);
  }
  return colors;
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  let r, g, b;

  if (s === 0) {
    r = g = b = l; // Grayscale
  } else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };

    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }

  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

function rgbToHex(r: number, g: number, b: number): string {
  return `#${((1 << 24) | (r << 16) | (g << 8) | b)
    .toString(16)
    .slice(1)
    .toUpperCase()}`;
}

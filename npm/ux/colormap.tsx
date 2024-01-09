// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// essentially borrowed from https://www.npmjs.com/package/colormap
// source are also available at https://github.com/bpostlethwaite/colormap

export function ColorMap(colorsCount: number): string[] {
  if (colorsCount <= predefinedColors.length) {
    return predefinedColors.slice(0, colorsCount);
  } else {
    const cmap = jetColorMap;
    const alpha = 1;

    // map index points from 0..1 to 0..n-1
    const indicies = cmap.map(function (c) {
      return Math.round(c.index * colorsCount);
    });

    const steps = cmap.map((c, i) => {
      const rgba = cmap[i].rgb.slice();

      // if user supplies their own map use it
      if (rgba.length === 4 && rgba[3] >= 0 && rgba[3] <= 1) {
        return rgba;
      }
      rgba[3] = alpha;

      return rgba;
    });

    /*
     * map increasing linear values between indicies to
     * linear steps in colorvalues
     */
    const colors = [];
    for (let i = 0; i < indicies.length - 1; ++i) {
      const nsteps = indicies[i + 1] - indicies[i];
      const fromrgba = steps[i];
      const torgba = steps[i + 1];

      for (let j = 0; j < nsteps; j++) {
        const amt = j / nsteps;
        colors.push([
          Math.round(linear(fromrgba[0], torgba[0], amt)),
          Math.round(linear(fromrgba[1], torgba[1], amt)),
          Math.round(linear(fromrgba[2], torgba[2], amt)),
          linear(fromrgba[3], torgba[3], amt),
        ]);
      }
    }

    //add 1 step as last value
    colors.push(cmap[cmap.length - 1].rgb.concat(alpha));

    return colors.map(rgb2hex);
  }
}

function linear(a: number, b: number, t: number) {
  return a + (b - a) * t;
}

function rgb2hex(rgba: number[]): string {
  let hex = "#";
  for (let i = 0; i < 3; ++i) {
    const dig = rgba[i].toString(16);
    hex += ("00" + dig).slice(dig.length);
  }
  return hex;
}

const predefinedColors = [
  "#FF0000", // Red
  "#0000FF", // Blue
  "#00FF00", // Green
  "#800080", // Purple
  "#FFA500", // Orange
  "#008080", // Teal
  "#FFC0CB", // Pink
  "#FFFF00", // Yellow
  "#A52A2A", // Brown
  "#00FFFF", // Cyan
];

// https://github.com/bpostlethwaite/colormap/blob/master/colorScale.js
const jetColorMap = [
  { index: 0, rgb: [0, 0, 131] },
  { index: 0.125, rgb: [0, 60, 170] },
  { index: 0.375, rgb: [5, 255, 255] },
  { index: 0.625, rgb: [255, 255, 0] },
  { index: 0.875, rgb: [250, 0, 0] },
  { index: 1, rgb: [128, 0, 0] },
];

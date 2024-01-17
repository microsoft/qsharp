// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function formatThousandSepF64(val: number): string {
  // Assuming val is 92592.5925925926
  const formatted = val.toFixed(2);
  const [integerPart, fractionalPart] = formatted.split(".");

  return `${formatThousandSep(integerPart)}.${fractionalPart}`;
}

export function formatThousandSep(str: string | number): string {
  if (typeof str === "number") {
    str = str.toString();
  }
  const parts = [];

  for (let i = str.length; i > 0; i -= 3) {
    parts.unshift(str.slice(Math.max(0, i - 3), i));
  }

  return parts.join(",");
}

// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { formatThousandSep } from "./utils.js";

export type Tick = {
  value: number;
  label: string;
};

// Should be redesigned in case of localization
type TickTimeDefinition = { tick: Tick; plural: string };

export function CreateIntegerTicks(min: number, max: number): Tick[] {
  if (max < min || min <= 0) {
    return [];
  }

  const l = Math.round(10 ** Math.floor(Math.log10(min)));
  const r = Math.round(10 ** Math.ceil(Math.log10(max)));

  const result: Tick[] = [];

  for (let i = l; i <= r; i *= 10) {
    if (i >= min && i <= max) {
      result.push({ value: i, label: formatThousandSep(i) });
    }
  }
  if (result.length == 0) {
    let step = l;
    while (step >= 1) {
      let i = Math.ceil(min / step) * step;
      while (i <= max) {
        result.push({ value: i, label: formatThousandSep(i) });
        i += step;
      }
      if (result.length > 0) {
        break;
      }
      step /= 10;
    }
  }

  return result;
}

const predefinedTimeTicks: TickTimeDefinition[] = [
  { tick: { value: 1, label: "nanosecond" }, plural: "nanoseconds" },
  { tick: { value: 1e3, label: "microsecond" }, plural: "microseconds" },
  { tick: { value: 1e6, label: "millisecond" }, plural: "milliseconds" },
  { tick: { value: 1e9, label: "second" }, plural: "seconds" },
  { tick: { value: 6e10, label: "minute" }, plural: "minutes" },
  { tick: { value: 3.6e12, label: "hour" }, plural: "hours" },
  { tick: { value: 8.64e13, label: "day" }, plural: "days" },
  { tick: { value: 6.048e14, label: "week" }, plural: "weeks" },
  { tick: { value: 2.592e15, label: "month" }, plural: "months" },
  { tick: { value: 3.1536e16, label: "year" }, plural: "years" },
  { tick: { value: 3.1536e17, label: "decade" }, plural: "decades" },
  { tick: { value: 3.1536e18, label: "century" }, plural: "centuries" },
];

export function CreateTimeTicks(min: number, max: number): Tick[] {
  if (max < min || min <= 0) {
    return [];
  }

  let l = 0;
  while (
    l < predefinedTimeTicks.length &&
    predefinedTimeTicks[l].tick.value <= min
  ) {
    l++;
  }

  let r = l;
  if (l > 0) {
    l--;
  }

  if (r >= predefinedTimeTicks.length) {
    r = predefinedTimeTicks.length - 1;
  }

  while (
    r < predefinedTimeTicks.length - 1 &&
    predefinedTimeTicks[r].tick.value <= max
  ) {
    r++;
  }

  const result: Tick[] = [];

  for (let i = l; i <= r; i++) {
    if (
      predefinedTimeTicks[i].tick.value >= min &&
      predefinedTimeTicks[i].tick.value <= max
    ) {
      result.push(predefinedTimeTicks[i].tick);
    }
  }

  if (result.length == 0) {
    if (l < predefinedTimeTicks.length - 1) {
      let coeff =
        10 **
        Math.floor(
          Math.log10(
            predefinedTimeTicks[l + 1].tick.value /
              predefinedTimeTicks[l].tick.value,
          ),
        );
      do {
        let i = 1;
        let val = 0;
        do {
          val = predefinedTimeTicks[l].tick.value * i * coeff;
          if (val >= min && val <= max) {
            result.push({
              value: val,
              label:
                (i * coeff).toString() + " " + predefinedTimeTicks[l].plural,
            });
          }
          i++;
        } while (
          // just a single tick in case of 1, 10 or 100
          (i > 2 || result.length < 1) &&
          val < predefinedTimeTicks[l + 1].tick.value
        );
        coeff /= 10;
      } while (result.length < 1 && coeff >= 1);
    } else {
      // > century
      let l = predefinedTimeTicks.length - 1;
      do {
        let coeff =
          10 ** Math.floor(Math.log10(max / predefinedTimeTicks[l].tick.value));
        do {
          let i = 1;
          let val = 0;
          do {
            val = predefinedTimeTicks[l].tick.value * i * coeff;
            if (val >= min && val <= max) {
              result.push({
                value: val,
                label:
                  (i * coeff).toString() + " " + predefinedTimeTicks[l].plural,
              });
            }
            i++;
          } while (
            // just a single tick in case of 1, 10 or 100
            (i > 2 || result.length < 1) &&
            val < max
          );
          coeff /= 10;
        } while (result.length < 1 && coeff >= 1);
        l--;
      } while (result.length < 1 && l >= 0);
    }
  }

  return result;
}

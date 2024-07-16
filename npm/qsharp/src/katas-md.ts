// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// IMPORTANT: This file is kept stand-alone from the general katas.ts so
// that the Markdown version of the bundle is not pulled into any packages
// when not needed, thus reducing bundle sizes for the production site.

import { default as katasContent } from "./katas-content.generated.md.js";

import type { Kata, Exercise } from "./katas.js";

export async function getAllKatas(
  options: { includeUnpublished?: boolean } = { includeUnpublished: false },
): Promise<Kata[]> {
  return katasContent.katas.filter(
    (k) => options.includeUnpublished || k.published,
  ) as Kata[];
}

export async function getExerciseSources(
  exercise: Exercise,
): Promise<string[]> {
  return katasContent.globalCodeSources
    .filter((source) => exercise.sourceIds.indexOf(source.id) > -1)
    .map((source) => source.code);
}

export type { Exercise, Kata };

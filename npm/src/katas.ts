// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { default as katasContent } from "./katas-content.generated.js";

// TODO: Remove.
export type Example = {
  type: "example";
  id: string;
  code: string;
};

export type QSharp = {
  type: "qsharp";
  sourceId: string;
  code: string;
};

export type Text = {
  type: "text";
  contentAsHtml: string;
  contentAsMarkdown: string;
};

export type LessonItem = QSharp | Text;

export type Lesson = {
  type: "reading";
  id: string;
  sections: LessonItem[];
};

export type Exercise = {
  type: "exercise";
  id: string;
  // TODO: fields that represent Q# code should be of QSharp type.
  codeDependencies: string[]; // Rename to just dependencies.
  verificationCode: string; // Rename to just verification.
  placeholderCode: string; // Rename to placeholder.
  solutionAsHtml: string;
  solutionAsMarkdown: string;
};

export type KataSection = Example | Exercise | Text;

export type Kata = {
  id: string;
  title: string;
  sections: KataSection[];
};

export async function getAllKatas(): Promise<Kata[]> {
  return katasContent.katas as Kata[];
}

export async function getKata(id: string): Promise<Kata> {
  const katas = await getAllKatas();
  return (
    katas.find((k) => k.id === id) ||
    Promise.reject(`Failed to get kata with id: ${id}`)
  );
}

export async function getExerciseDependencies(
  exercise: Exercise
): Promise<string[]> {
  return katasContent.globalCodeSources
    .filter((source) => exercise.codeDependencies.indexOf(source.name) > -1)
    .map((source) => source.contents);
}

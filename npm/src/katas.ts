// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { default as katasContent } from "./katas-content.generated.js";

export type Example = {
  type: "example";
  id: string;
  code: string;
};

export type Text = {
  type: "text";
  contentAsHtml: string;
  contentAsMarkdown: string;
};

export type TextContent = {
  type: "text-content";
  asHtml: string;
  asMarkdown: string;
};

export type Solution = {
  type: "solution";
  id: string;
  code: string;
};

export type ExplainedSolutionItem = Example | Solution | TextContent;

export type ExplainedSolution = {
  type: "explained-solution";
  items: ExplainedSolutionItem[];
};

export type Exercise = {
  type: "exercise";
  id: string;
  title: string;
  description: TextContent;
  sourceIds: string[];
  placeholderCode: string;
  explainedSolution: ExplainedSolution;
};

export type LessonItem = Example | TextContent;

export type Lesson = {
  type: "lesson";
  id: string;
  title: string;
  items: LessonItem[];
};

export type Question = {
  type: "question";
  id: string;
  title: string;
  description: TextContent;
  answer: TextContent;
};

export type KataSection = Exercise | Lesson | Question;

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

export async function getExerciseSources(
  exercise: Exercise
): Promise<string[]> {
  return katasContent.globalCodeSources
    .filter((source) => exercise.sourceIds.indexOf(source.id) > -1)
    .map((source) => source.code);
}

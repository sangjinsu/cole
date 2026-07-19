// @ts-expect-error Vitest runs in Node, while the application tsconfig intentionally omits Node types.
import { readFileSync } from "node:fs";

declare const process: { cwd: () => string };

const styles = readFileSync(`${process.cwd()}/src/index.css`, "utf8");

test("keeps Add child available in narrow layouts", () => {
  expect(styles).not.toMatch(
    /\.row-actions\s+\.icon-button:first-child\s*\{[^}]*display:\s*none/,
  );
});

test("stacks analysis inside the narrow viewport without hiding zoom controls", () => {
  expect(styles).not.toMatch(/\.analysis-canvas-scale\s*\{[^}]*min-width:\s*(680|780)px/);
  expect(styles).toMatch(
    /@media \(max-width:\s*760px\)[\s\S]*\.analysis-layout\s*\{[^}]*grid-template-columns:\s*1fr/,
  );
  expect(styles).not.toMatch(
    /@media \(max-width:\s*760px\)[\s\S]*\.zoom-controls\s*\{[^}]*display:\s*none/,
  );
});

import { QueryClient } from "@tanstack/react-query";
import type { AnalysisSnapshot, ChecklistTree } from "../../types/cole";
import { latestAnalysisQueryKey, promoteFreshAnalysisSnapshot } from "./cache";

const tree: ChecklistTree = {
  checklist: {
    id: "default",
    title: "Today's checklist",
    revision: 7,
    checklistHash: "current-hash",
    updatedAt: "2026-07-11T04:00:00Z",
  },
  nodes: [],
};

const currentSnapshot: AnalysisSnapshot = {
  id: "current",
  checklistId: "default",
  checklistRevision: 7,
  checklistHash: "current-hash",
  taskIds: [],
  instructionHash: "instruction-hash",
  requestHash: "request-hash",
  provider: "deterministic",
  requestedModel: "gpt-5.6",
  resolvedModel: null,
  fallbackReason: "missing_credential",
  openuiResponse: "",
  generatedAt: "2026-07-11T04:01:00Z",
  state: "fresh",
  result: { summary: "Current", groups: [] },
};

test("does not promote a late stale response over the latest snapshot", () => {
  const client = new QueryClient();
  const staleResponse = {
    ...currentSnapshot,
    id: "late-stale",
    checklistRevision: 6,
    checklistHash: "old-hash",
  };
  client.setQueryData(["default-checklist"], tree);
  client.setQueryData(latestAnalysisQueryKey("default"), currentSnapshot);

  const promoted = promoteFreshAnalysisSnapshot(client, staleResponse);

  expect(promoted).toBe(false);
  expect(client.getQueryData(latestAnalysisQueryKey("default"))).toEqual(currentSnapshot);
});

test("promotes a response only while its revision and hash still match the checklist", () => {
  const client = new QueryClient();
  client.setQueryData(["default-checklist"], tree);

  const promoted = promoteFreshAnalysisSnapshot(client, currentSnapshot);

  expect(promoted).toBe(true);
  expect(client.getQueryData(latestAnalysisQueryKey("default"))).toEqual(currentSnapshot);
});

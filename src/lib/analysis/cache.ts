import type { QueryClient, QueryKey } from "@tanstack/react-query";
import type { AnalysisSnapshot, ChecklistTree } from "../../types/cole";

export const defaultChecklistQueryKey = ["default-checklist"] as const;

export function latestAnalysisQueryKey(checklistId: string): QueryKey {
  return ["latest-analysis", checklistId] as const;
}

export function promoteFreshAnalysisSnapshot(
  client: QueryClient,
  snapshot: AnalysisSnapshot,
): boolean {
  const currentTree = client.getQueryData<ChecklistTree>(defaultChecklistQueryKey);
  const isFresh =
    currentTree?.checklist.id === snapshot.checklistId &&
    currentTree.checklist.revision === snapshot.checklistRevision &&
    currentTree.checklist.checklistHash === snapshot.checklistHash;

  if (!isFresh) return false;

  client.setQueryData(latestAnalysisQueryKey(snapshot.checklistId), snapshot);
  return true;
}

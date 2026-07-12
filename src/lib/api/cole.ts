import { invoke } from "@tauri-apps/api/core";
import type {
  AnalysisSnapshot,
  AnalyzeChecklistInput,
  ArchiveChecklistNodeInput,
  ChecklistTree,
  CreateChecklistNodeInput,
  OpenAiConnectionResult,
  OpenAiCredentialStatus,
  RenameChecklistNodeInput,
  SetTaskCheckedInput,
  SetTaskEstimateInput,
} from "../../types/cole";

const inTauriRuntime =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!inTauriRuntime) {
    throw new Error("Cole desktop commands are available only inside Tauri.");
  }

  return invoke<T>(command, args);
}

export function getDefaultChecklist(): Promise<ChecklistTree> {
  return call<ChecklistTree>("get_default_checklist");
}

export function createChecklistNode(
  input: CreateChecklistNodeInput,
): Promise<ChecklistTree> {
  return call<ChecklistTree>("create_checklist_node", { input });
}

export function renameChecklistNode(
  input: RenameChecklistNodeInput,
): Promise<ChecklistTree> {
  return call<ChecklistTree>("rename_checklist_node", { input });
}

export function setTaskChecked(input: SetTaskCheckedInput): Promise<ChecklistTree> {
  return call<ChecklistTree>("set_task_checked", { input });
}

export function setTaskEstimate(input: SetTaskEstimateInput): Promise<ChecklistTree> {
  return call<ChecklistTree>("set_task_estimate", { input });
}

export function archiveChecklistNode(
  input: ArchiveChecklistNodeInput,
): Promise<ChecklistTree> {
  return call<ChecklistTree>("archive_checklist_node", { input });
}

export function analyzeChecklist(input: AnalyzeChecklistInput): Promise<AnalysisSnapshot> {
  return call<AnalysisSnapshot>("analyze_checklist", { input });
}

export function getLatestAnalysisSnapshot(
  checklistId: string,
): Promise<AnalysisSnapshot | null> {
  return call<AnalysisSnapshot | null>("get_latest_analysis_snapshot", { checklistId });
}

export function getAnalysisSnapshot(snapshotId: string): Promise<AnalysisSnapshot> {
  return call<AnalysisSnapshot>("get_analysis_snapshot", { snapshotId });
}

export function setOpenAiApiKey(apiKey: string): Promise<OpenAiCredentialStatus> {
  return call<OpenAiCredentialStatus>("set_openai_api_key", { apiKey });
}

export function getOpenAiCredentialStatus(): Promise<OpenAiCredentialStatus> {
  return call<OpenAiCredentialStatus>("get_openai_credential_status");
}

export function deleteOpenAiApiKey(): Promise<OpenAiCredentialStatus> {
  return call<OpenAiCredentialStatus>("delete_openai_api_key");
}

export function testOpenAiConnection(): Promise<OpenAiConnectionResult> {
  return call<OpenAiConnectionResult>("test_openai_connection");
}

export function isDesktopRuntime(): boolean {
  return inTauriRuntime;
}

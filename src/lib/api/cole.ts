import { invoke } from "@tauri-apps/api/core";
import type {
  CreateObsidianSourceInput,
  RecommendationFlow,
  Source,
  SyncResult,
  Task,
} from "../../types/cole";

const inTauriRuntime =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!inTauriRuntime) {
    throw new Error("Cole desktop commands are available only inside Tauri.");
  }

  return invoke<T>(command, args);
}

export async function listSources(): Promise<Source[]> {
  return call<Source[]>("list_sources");
}

export async function createObsidianSource(
  input: CreateObsidianSourceInput,
): Promise<Source> {
  return call<Source>("create_obsidian_source", { input });
}

export async function syncObsidianSource(sourceId: string): Promise<SyncResult> {
  return call<SyncResult>("sync_obsidian_source", { sourceId });
}

export async function listTasks(): Promise<Task[]> {
  return call<Task[]>("list_tasks");
}

export async function getRecommendationFlow(): Promise<RecommendationFlow> {
  return call<RecommendationFlow>("get_recommendation_flow");
}

export async function markTaskDoneLocal(taskId: string): Promise<Task> {
  return call<Task>("mark_task_done_local", { taskId });
}

export function isDesktopRuntime(): boolean {
  return inTauriRuntime;
}
